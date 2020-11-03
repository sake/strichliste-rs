use std::net::{SocketAddr, ToSocketAddrs};
use sqlx::sqlite::{SqliteRow, SqlitePool};
use sqlx::{Pool, Row};
use warp::Filter;

#[tokio::main]
async fn main() {
    let db = match open_db().await {
        Ok(db) => db,
        Err(e) => panic!("{}", e),
    };
    match migrate_db(&db).await {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };

    // TODO: add error handling and read addr from env var
    let mut addr_iter = "[::]:3030".to_socket_addrs().unwrap();
    let addr = addr_iter.next().unwrap();

    start_webserver(addr).await;
}


async fn open_db() -> Result<SqlitePool, sqlx::Error> {
    let db = Pool::new("sqlite:/tmp/strichliste.sqlite");
    return db.await;
}

async fn migrate_db(db: &SqlitePool) -> Result<(), sqlx::Error> {
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

    // db.execute("CASE WHEN SELECT num FROM version WHERE num = 1 THEN
    //  CREATE TABLE version (num ")?;

    return Ok(());
}

async fn start_webserver(addr: SocketAddr) {
    // see next link how to add apis
    // https://blog.logrocket.com/creating-a-rest-api-in-rust-with-warp/

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(addr).await;
}
