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
        let user = user_db::get_user(db, settings, &parent.user_id)
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

pub async fn get_transaction(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    transact_id: &i32,
) -> Result<model::TransactionObject> {
    let transaction_entity = sqlx::query_as::<_, model::TransactionEntity>(
		"SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions
		WHERE id = ?"
	)
	.bind(transact_id)
	.fetch_one(db).await?;

    let user = user_db::get_user(db, settings, &transaction_entity.user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    return Ok(model::TransactionObject {
        entity: transaction_entity,
        user,
        article: None,
        recipient_transaction: None,
        sender_transaction: None,
    });
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
    mut article: model::ArticleObject,
    comment: Option<&str>,
) -> Result<model::TransactionObject> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
		"UPDATE user SET balance = balance + ?
        WHERE id = ?;
        
        UPDATE article SET usage_count = usage_count + 1
        WHERE id = ?;
		
		INSERT INTO transactions (user_id, article_id, quantity, comment, amount, deleted, created)
		VALUES (?, ?, ?, ?, ?, FALSE, datetime('now'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions WHERE id = last_insert_rowid();"
	)
	.bind(amount)
    .bind(user.id)
    .bind(article.entity.id)
    .bind(user.id)
    .bind(article.entity.id)
    .bind(quantity)
	.bind(comment)
	.bind(amount)
	.fetch_one(db).await?;

    // correct entity objects
    user.balance += amount;
    article.entity.usage_count += 1;

    return Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: Some(article),
        sender_transaction: None,
        recipient_transaction: None,
    });
}

pub async fn add_transaction_with_recipient(
    db: &SqlitePool,
    mut user: model::UserEntity,
    amount: &i32,
    mut recipient: model::UserEntity,
    comment: Option<&str>,
) -> Result<model::TransactionObject> {
    let mut result = sqlx::query_as::<_, model::TransactionEntity>(
		"UPDATE user SET balance = balance + ?
        WHERE id = ?;
        UPDATE user SET balance = (balance + ?) * -1
		WHERE id = ?;
        
        -- sender transaction
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
        VALUES (?, ?, ?, FALSE, datetime('now'));
        -- recipient transaction
		INSERT INTO transactions (user_id, recipient_transaction_id, comment, amount, deleted, created)
		VALUES (?, last_insert_rowid(), ?, ? * -1, FALSE, datetime('now'));
        -- update sender transaction reference
        UPDATE transactions SET sender_transaction_id = last_insert_rowid()
        WHERE id IN (SELECT recipient_transaction_id FROM transactions WHERE id = last_insert_rowid());
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions WHERE id = last_insert_rowid() OR sender_transaction_id = last_insert_rowid()
		ORDER BY id ASC;"
    )
    // update user
	.bind(amount)
	.bind(user.id)
	.bind(amount)
    .bind(recipient.id)
    // insert sender transaction
	.bind(user.id)
	.bind(comment)
    .bind(amount)
        // insert recipient transaction
	.bind(recipient.id)
	.bind(comment)
	.bind(amount)
	.fetch_all(db).await?;

    // correct entity object
    user.balance += amount;
    recipient.balance -= amount;

    match (result.pop(), result.pop()) {
        (Some(sender_tx), Some(recep_tx)) => Ok(model::TransactionObject {
            entity: sender_tx,
            user: user,
            article: None,
            sender_transaction: None,
            recipient_transaction: Some(Box::new(model::TransactionObject {
                entity: recep_tx,
                user: recipient,
                article: None,
                sender_transaction: None,
                recipient_transaction: None,
            })),
        }),
        // TODO: should be something like internal server error or unknown error
        _ => Err(sqlx::Error::ColumnNotFound("unknown".to_string())),
    }
}
