use sqlx::{sqlite::SqlitePool, Result};

use crate::{model, settings::StrichlisteSetting, user_db};

pub async fn get_transactions(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    user_id: &i32,
    limit: &i32,
    offset: &i32,
) -> Result<Vec<model::TransactionObject>> {
    let transaction_entities_result = sqlx::query_as::<_, model::TransactionEntity>(
		"SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions
		WHERE user_id = ?
		ORDER BY created DESC LIMIT ? OFFSET ?"
	)
	.bind(user_id)
	.bind(limit)
	.bind(offset)
	.fetch_all(db).await?;

    let mut result = Vec::new();
    for parent in transaction_entities_result {
        let user = user_db::get_user(db, settings, parent.user_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        let o = model::TransactionObject {
            entity: parent,
            user,
            article: None,
            recipient_transaction: None,
            sender_transaction: None,
        };
        result.push(o);
    }

    return Ok(result);
}

pub async fn num_active(db: &SqlitePool, user_id: Option<i32>) -> Result<i32> {
    let count_result = sqlx::query_scalar::<_, i32>(
        "SELECT count(*)
		FROM transactions
		WHERE CASE WHEN ? NOT NULL THEN user_id = ? ELSE TRUE END;",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_one(db);

    return count_result.await;
}

pub async fn add_transaction_with_value(
    db: &SqlitePool,
    mut user: model::UserEntity,
    amount: &i32,
    comment: Option<&str>,
) -> Result<model::TransactionObject> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
		"UPDATE user SET balance = balance + ?
		WHERE id = ?;
		
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
		VALUES (?, ?, ?, FALSE, datetime('now'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions WHERE id = last_insert_rowid();"
	)
	.bind(amount)
	.bind(user.id)
	.bind(user.id)
	.bind(comment)
	.bind(amount)
	.fetch_one(db).await?;

    // correct entity object
    user.balance += amount;

    return Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: None,
        sender_transaction: None,
        recipient_transaction: None,
    });
}

pub async fn add_transaction_with_article(
    db: &SqlitePool,
    mut user: model::UserEntity,
    quantity: &i32,
    amount: &i32,
    article: model::ArticleObject,
    comment: Option<&str>,
) -> Result<model::TransactionObject> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
		"UPDATE user SET balance = balance + ?
		WHERE id = ?;
		
		INSERT INTO transactions (user_id, article_id, quantity, comment, amount, deleted, created)
		VALUES (?, ?, ?, ?, ?, FALSE, datetime('now'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions WHERE id = last_insert_rowid();"
	)
	.bind(amount)
	.bind(user.id)
    .bind(user.id)
    .bind(article.entity.id)
    .bind(quantity)
	.bind(comment)
	.bind(amount)
	.fetch_one(db).await?;

    // correct entity object
    user.balance += amount;

    return Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: Some(article),
        sender_transaction: None,
        recipient_transaction: None,
    });
}

// pub async fn add_transaction_with_recipient(
// 	db: &SqlitePool,
// 	settings: &StrichlisteSetting,
// 	user: model::UserEntity,
// 	amount: &i32,
// 	recipient: Option<model::UserEntity>,
// ) -> Result<model::TransactionObject> {

// }
