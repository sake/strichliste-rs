use std::time::Duration;

use log::info;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow},
    Row,
};

pub async fn open_db(db_file: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(db_file)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_millis(5000))
        .create_if_missing(true);
    let db = SqlitePoolOptions::new()
        //.max_connections(1)
        .connect_with(opts)
        .await?;

    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&db)
        .await?;

    return Ok(db);
}

pub async fn migrate_db(db: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Checking DB migration ...");

    // get latest version number
    let mut cur_version: i32 = sqlx::query("PRAGMA user_version;")
        .map(|row: SqliteRow| row.get(0))
        .fetch_one(db)
        .await?;

    if cur_version == 0 {
        cur_version += 1;
        info!("Running migration #{}", cur_version);

        let mut tx = db.begin().await?;
        sqlx::query("
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
                    --CREATE UNIQUE INDEX uniq_active_barcode ON article (barcode) where active is TRUE;
                    CREATE TRIGGER trgr_update_no_inactive_article BEFORE INSERT ON article
                        WHEN NEW.precursor_id NOT NULL AND EXISTS (SELECT * from article WHERE id = NEW.precursor_id AND active IS FALSE)
                        BEGIN
                            SELECT RAISE(FAIL, 'Updating inactive article is not allowed.');
                        END;
                    CREATE TRIGGER trgr_update_unique_name_barcode BEFORE INSERT ON article
                        WHEN EXISTS (SELECT 1 from article WHERE active IS TRUE AND id != NEW.precursor_id AND (name = NEW.name OR barcode = NEW.barcode))
                        BEGIN
                            SELECT RAISE(FAIL, 'Name or barcode is not unique.');
                        END;

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
                    CREATE INDEX idx_transaction_sendertxid ON transactions (sender_transaction_id);
                    CREATE INDEX idx_transaction_recipienttxid ON transactions (recipient_transaction_id);

                    PRAGMA user_version = 1;")
                    .execute(&mut tx).await?;
        tx.commit().await?;
    }

    return Ok(());
}
