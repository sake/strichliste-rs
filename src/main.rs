use serde::{Deserialize, Serialize};
use sqlx::prelude::SqliteQueryAs;
use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::{Pool, Row};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use warp::Filter;

#[tokio::main]
async fn main() {
    let db_file = env_or("DB_FILE", "/tmp/strichliste.sqlite");
    let db = match open_db(db_file.as_str()).await {
        Ok(db) => db,
        Err(e) => panic!("{}", e),
    };
    match migrate_db(&db).await {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };

    // TODO: add error handling
    let addr_str = env_or("BIND_ADDRESS", "[::]:3030");
    let mut addr_iter = addr_str.to_socket_addrs().unwrap();
    let addr = addr_iter.next().unwrap();

    start_webserver(addr, db).await;
}

fn env_or(key: &str, default: &str) -> String {
    return env::var(key).ok().unwrap_or(default.to_string());
}

async fn open_db(db_file: &str) -> Result<SqlitePool, sqlx::Error> {
    let db_string = format!("sqlite:{}", db_file);
    let db = Pool::new(db_string.as_str());
    return db.await;
}

async fn migrate_db(db: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("Checking DB migration ...");

    // enable WAL mode
    //db.execute("PRAGMA journal_mode=WAL;")?;

    // make sure the version table exists
    sqlx::query("CREATE TABLE IF NOT EXISTS version (num INTEGER PRIMARY KEY);")
        .execute(db)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO version VALUES(0);")
        .execute(db)
        .await?;

    // get latest version number
    let cur_version: i32 = sqlx::query("SELECT num FROM version ORDER BY num DESC LIMIT 1")
        .map(|row: SqliteRow| row.get(0))
        .fetch_one(db)
        .await?;

    if cur_version == 0 {
        println!("Running migration #{}", cur_version + 1);
        sqlx::query("BEGIN TRANSACTION;

                    CREATE TABLE article (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        precursor_id INTEGER DEFAULT NULL,
                        name VARCHAR(255) NOT NULL,
                        barcode VARCHAR(32) DEFAULT NULL,
                        amount INTEGER NOT NULL,
                        active BOOLEAN NOT NULL,
                        created DATETIME NOT NULL,
                        usage_count INTEGER NOT NULL,
                        CONSTRAINT uniq_precursor UNIQUE (precursor_id),
                        CONSTRAINT fk_article_precursor_article_id FOREIGN KEY (precursor_id) REFERENCES article (id)
                    );

                    CREATE TABLE user (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name VARCHAR(64) NOT NULL,
                        email VARCHAR(255) DEFAULT NULL,
                        balance INTEGER NOT NULL,
                        disabled BOOLEAN NOT NULL,
                        created DATETIME NOT NULL,
                        updated DATETIME DEFAULT NULL,
                        CONSTRAINT uniq_user_name UNIQUE (name)
                    );
                    CREATE INDEX idx_user_disabled_updated ON user (disabled, updated);

                    CREATE TABLE transactions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL,
                        article_id INTEGER DEFAULT NULL,
                        recipient_transaction_id INTEGER DEFAULT NULL,
                        sender_transaction_id INTEGER DEFAULT NULL,
                        quantity INTEGER DEFAULT NULL,
                        comment VARCHAR(255) DEFAULT NULL,
                        amount INTEGER NOT NULL,
                        deleted BOOLEAN NOT NULL,
                        created DATETIME NOT NULL,
                        CONSTRAINT uniq_transaction_recepient UNIQUE (recipient_transaction_id),
                        CONSTRAINT uniq_transaction_sender UNIQUE (sender_transaction_id),
                        CONSTRAINT fk_transaction_article_id FOREIGN KEY (article_id) REFERENCES article (id),
                        CONSTRAINT fk_transaction_recipient_id FOREIGN KEY (recipient_transaction_id) REFERENCES transactions (id) ON DELETE CASCADE,
                        CONSTRAINT fk_transaction_user_id FOREIGN KEY (user_id) REFERENCES user (id),
                        CONSTRAINT fk_transaction_sender_id FOREIGN KEY (sender_transaction_id) REFERENCES transactions (id) ON DELETE CASCADE
                    );
                    CREATE INDEX idx_transaction_userid ON transactions (user_id);
                    CREATE INDEX idx_transaction_articleid ON transactions (article_id);

                    INSERT INTO version VALUES(1);

                    END TRANSACTION;")
                    .execute(db).await?;
    }

    return Ok(());
}

#[derive(Debug, sqlx::FromRow)]
struct UserEntity {
    id: i32,
    name: String,
    email: Option<String>,
    balance: i32,
    disabled: bool,
    created: String,
    updated: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
struct UserResponse {
    id: i32,
    name: String,
    email: Option<String>,
    balance: i32,
    #[serde(rename(serialize = "isActive", deserialize = "isActive"))]
    active: bool,
    #[serde(rename(serialize = "isDisabled", deserialize = "isDisabled"))]
    disabled: bool,
    created: String,
    updated: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UsersResponse {
    users: Vec<UserResponse>,
}

async fn start_webserver(addr: SocketAddr, db: SqlitePool) {
    println!("Starting webserver binding to {} ...", addr);

    // see next link how to add apis
    // https://blog.logrocket.com/creating-a-rest-api-in-rust-with-warp/

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let settings = warp::get()
        .and(warp::path!("settings"))
        .and_then(get_settings);
    let users = warp::get()
        .and(warp::path!("user"))
        .and(warp::query::<HashMap<String, String>>())
        .and(with_db(db))
        .and_then(get_users);

    let api = warp::path("api").and(hello.or(settings).or(users));

    warp::serve(api).run(addr).await;
}

fn with_db(
    db_pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

async fn get_settings() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let settings = default_settings();
    return Ok(Box::new(warp::reply::json(&settings)));
}

async fn get_users(
    query: HashMap<String, String>,
    db: SqlitePool,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let deleted: bool = query
        .get("deleted")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(false);

    let stale_period = ms_converter::ms("10 day").unwrap() * 1000;

    let user_entities_result = sqlx::query_as::<_, UserEntity>(
        "SELECT id, name, email, balance, disabled, created, updated FROM user WHERE disabled IS ?",
    )
    .bind(deleted.to_string())
    .fetch_all(&db);

    let result: UsersResponse;
    match user_entities_result.await {
        Ok(user_entities) => {
            let user_responses = user_entities
                .iter()
                .map(|v| UserResponse {
                    id: v.id,
                    name: v.name.clone(),
                    email: v.email.clone(),
                    balance: v.balance,
                    active: true,
                    disabled: v.disabled,
                    created: v.created.clone(),
                    updated: v.updated.clone(),
                })
                .collect();
            result = UsersResponse {
                users: user_responses,
            };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        Err(e) => {
            println!("Failed to query user table. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SettingsWrapper {
    parameters: Settings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Settings {
    strichliste: StrichlisteSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct StrichlisteSetting {
    article: ArticleSettings,
    common: CommonSettings,
    paypal: PaypalSetting,
    user: UserSetting,
    i18n: I18nSetting,
    account: AccountSetting,
    payment: PaymentSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ArticleSettings {
    enabled: bool,
    #[serde(rename(serialize = "autoOpen", deserialize = "autoOpen"))]
    auto_open: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CommonSettings {
    #[serde(rename(serialize = "idleTimeout", deserialize = "idleTimeout"))]
    idle_timeout: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PaypalSetting {
    enabled: bool,
    recipient: String,
    fee: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserSetting {
    #[serde(rename(serialize = "stalePeriod", deserialize = "stalePeriod"))]
    stale_period: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct I18nSetting {
    #[serde(rename(serialize = "dateFormat", deserialize = "dateFormat"))]
    date_format: String,
    timezone: String,
    language: String,
    currency: CurrencySetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CurrencySetting {
    name: String,
    symbol: String,
    alpha3: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AccountSetting {
    boundary: BoundarySetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BoundarySetting {
    upper: i32,
    lower: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PaymentSetting {
    undo: UndoSetting,
    boundary: BoundarySetting,
    transactions: TransactionSetting,
    #[serde(rename(serialize = "splitInvoice", deserialize = "splitInvoice"))]
    split_invoice: SplitInvoiceSetting,
    deposit: DepositSetting,
    dispense: DepositSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UndoSetting {
    enabled: bool,
    delete: bool,
    timeout: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TransactionSetting {
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SplitInvoiceSetting {
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DepositSetting {
    enabled: bool,
    custom: bool,
    steps: Vec<i32>,
}

fn default_settings() -> SettingsWrapper {
    return SettingsWrapper {
        parameters: Settings {
            strichliste: StrichlisteSetting {
                article: ArticleSettings {
                    enabled: true,
                    auto_open: false,
                },
                common: CommonSettings {
                    idle_timeout: 30000,
                },
                paypal: PaypalSetting {
                    enabled: false,
                    recipient: "foo@bar.de".to_string(),
                    fee: 0,
                },
                user: UserSetting {
                    stale_period: "10 day".to_string(),
                },
                i18n: I18nSetting {
                    date_format: "YYYY-MM-DD HH:mm:ss".to_string(),
                    timezone: "auto".to_string(),
                    language: "en".to_string(),
                    currency: CurrencySetting {
                        name: "Euro".to_string(),
                        symbol: "€".to_string(),
                        alpha3: "EUR".to_string(),
                    },
                },
                account: AccountSetting {
                    boundary: BoundarySetting {
                        upper: 20000,
                        lower: -20000,
                    },
                },
                payment: PaymentSetting {
                    undo: UndoSetting {
                        enabled: true,
                        delete: false,
                        timeout: "5 minute".to_string(),
                    },

                    boundary: BoundarySetting {
                        upper: 15000,
                        lower: -2000,
                    },

                    transactions: TransactionSetting { enabled: true },

                    split_invoice: SplitInvoiceSetting { enabled: true },

                    deposit: DepositSetting {
                        enabled: true,
                        custom: true,
                        steps: [50, 100, 200, 500, 1000].to_vec(),
                    },

                    dispense: DepositSetting {
                        enabled: true,
                        custom: true,
                        steps: [50, 100, 200, 500, 1000].to_vec(),
                    },
                },
            },
        },
    };
}