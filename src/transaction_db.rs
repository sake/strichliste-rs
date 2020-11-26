use sqlx::{sqlite::SqlitePool, Result};

use crate::{error::DbError, article_db, common, model, settings::StrichlisteSetting, user_db};

pub async fn get_transactions(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    user_id: &i32,
    limit: &i32,
    offset: &i32,
) -> std::result::Result<Vec<model::TransactionObject>, DbError> {
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
        let article = article_db::get_article(db, parent.article_id).await?;
        let recipient_tx =
            get_child_transaction(db, settings, parent.recipient_transaction_id.as_ref()).await?;
        let sender_tx =
            get_child_transaction(db, settings, parent.sender_transaction_id.as_ref()).await?;
        let o = model::TransactionObject {
            entity: parent,
            user,
            article: article.map(|a| *a),
            recipient: recipient_tx.map(|v| v.user),
            sender: sender_tx.map(|v| v.user),
        };
        result.push(o);
    }

    return Ok(result);
}

async fn get_child_transaction(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    transact_id: Option<&i32>,
) -> std::result::Result<Option<Box<model::TransactionObject>>, DbError> {
    match transact_id {
        Some(tx_id) => {
            let transaction_entity = sqlx::query_as::<_, model::TransactionEntity>(
		"SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions
        WHERE id = ?"
	)
	.bind(tx_id)
	.fetch_one(db).await?;

            match user_db::get_user(db, settings, &transaction_entity.user_id).await {
                Ok(Some(user)) => Ok(Some(Box::new(model::TransactionObject {
                    entity: transaction_entity,
                    user,
                    article: None,
                    recipient: None,
                    sender: None,
                }))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        }
        None => Ok(None),
    }
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
        "BEGIN TRANSACTION;
        UPDATE user SET balance = balance + ?, updated = datetime('now')
		WHERE id = ?;
		
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
		VALUES (?, ?, ?, FALSE, datetime('now'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions WHERE id = last_insert_rowid();
        END TRANSACTION;"
	)
	.bind(amount)
	.bind(user.id)
	.bind(user.id)
	.bind(comment)
	.bind(amount)
	.fetch_one(db).await?;

    // correct entity object
    user.balance += amount;
    user.updated = Some(common::cur_datetime_str());

    return Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: None,
        sender: None,
        recipient: None,
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
        "BEGIN TRANSACTION;
        UPDATE user SET balance = balance + ?, updated = datetime('now')
        WHERE id = ?;
        
        UPDATE article SET usage_count = usage_count + 1
        WHERE id = ?;
		
		INSERT INTO transactions (user_id, article_id, quantity, comment, amount, deleted, created)
		VALUES (?, ?, ?, ?, ?, FALSE, datetime('now'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions WHERE id = last_insert_rowid();
        END TRANSACTION;"
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
    user.updated = Some(common::cur_datetime_str());
    article.entity.usage_count += 1;

    return Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: Some(article),
        sender: None,
        recipient: None,
    });
}

pub async fn add_transaction_with_recipient(
    db: &SqlitePool,
    mut user: model::UserEntity,
    amount: &i32,
    mut recipient: model::UserEntity,
    comment: Option<&str>,
) -> Result<model::TransactionObject> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
        "BEGIN TRANSACTION;

        UPDATE user SET balance = balance + ?, updated = datetime('now')
        WHERE id = ?;
        UPDATE user SET balance = (balance + ?) * -1
		WHERE id = ?;
        
        -- sender transaction
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
        VALUES (?, ?, ?, FALSE, datetime('now'));
        -- recipient transaction
		INSERT INTO transactions (user_id, sender_transaction_id, comment, amount, deleted, created)
		VALUES (?, last_insert_rowid(), ?, ? * -1, FALSE, datetime('now'));
        -- update sender transaction reference
        UPDATE transactions SET recipient_transaction_id = last_insert_rowid()
        WHERE id IN (SELECT sender_transaction_id FROM transactions WHERE id = last_insert_rowid());
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions WHERE recipient_transaction_id = last_insert_rowid();
        
        END TRANSACTION;"
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
	.fetch_one(db).await?;

    // correct entity object
    user.balance += amount;
    user.updated = Some(common::cur_datetime_str());
    recipient.balance -= amount;

    Ok(model::TransactionObject {
        entity: result,
        user: user,
        article: None,
        sender: None,
        recipient: Some(recipient),
    })
}
