use sqlx::{sqlite::SqlitePool, Sqlite, Transaction};

use crate::{article_db, common, error::DbError, model, settings::StrichlisteSetting, user_db};

pub async fn get_transactions(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    user_id: &i32,
    limit: &i32,
    offset: &i32,
) -> std::result::Result<Vec<model::TransactionObject>, DbError> {
    let mut tx = db.begin().await?;
    let transaction_entities_result = sqlx::query_as::<_, model::TransactionEntity>(
		"SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
		FROM transactions
		WHERE user_id = ?
		ORDER BY created DESC LIMIT ? OFFSET ?"
	)
	.bind(user_id)
	.bind(limit)
	.bind(offset)
	.fetch_all(&mut tx).await?;

    let mut result = Vec::new();
    for parent in transaction_entities_result {
        let user = user_db::get_user_tx(&mut tx, settings, &parent.user_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        let article = article_db::get_article_tx(&mut tx, parent.article_id).await?;
        let recipient_tx =
            get_child_transaction_tx(&mut tx, settings, parent.recipient_transaction_id.as_ref())
                .await?;
        let sender_tx =
            get_child_transaction_tx(&mut tx, settings, parent.sender_transaction_id.as_ref())
                .await?;
        let o = model::TransactionObject {
            entity: parent,
            user,
            article: article.map(|a| *a),
            recipient: recipient_tx.map(|v| v.user),
            sender: sender_tx.map(|v| v.user),
        };
        result.push(o);
    }

    tx.commit().await?;

    return Ok(result);
}

async fn get_child_transaction_tx(
    tx: &mut Transaction<'static, Sqlite>,
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
	        .fetch_one(&mut *tx).await?;

            let user_entity = user_db::get_user_tx(&mut *tx, settings, &transaction_entity.user_id).await?;
            match user_entity {
                Some(user) => Ok(Some(Box::new(model::TransactionObject {
                    entity: transaction_entity,
                    user,
                    article: None,
                    recipient: None,
                    sender: None,
                }))),
                None => Ok(None),
            }
        }
        None => Ok(None),
    }
}

pub async fn num_active(
    db: &SqlitePool,
    user_id: Option<i32>,
) -> std::result::Result<i32, DbError> {
    let mut tx = db.begin().await?;
    let count_result = sqlx::query_scalar::<_, i32>(
        "SELECT count(*)
		FROM transactions
		WHERE CASE WHEN ? NOT NULL THEN user_id = ? ELSE TRUE END;",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    return Ok(count_result);
}

pub async fn add_transaction_with_value_tx(
    tx: &mut Transaction<'static, Sqlite>,
    mut user: model::UserEntity,
    amount: &i32,
    comment: Option<&str>,
) -> std::result::Result<model::TransactionObject, DbError> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
        "UPDATE user SET balance = balance + ?, updated = datetime('now', 'localtime')
		WHERE id = ?;
		
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
		VALUES (?, ?, ?, FALSE, datetime('now', 'localtime'));
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions WHERE id = last_insert_rowid();"
	)
	.bind(amount)
	.bind(user.id)
	.bind(user.id)
	.bind(comment)
	.bind(amount)
    .fetch_one(tx).await?;

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

pub async fn add_transaction_with_article_tx(
    tx: &mut Transaction<'static, Sqlite>,
    mut user: model::UserEntity,
    quantity: &i32,
    amount: &i32,
    mut article: model::ArticleObject,
    comment: Option<&str>,
) -> std::result::Result<model::TransactionObject, DbError> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
        "UPDATE user SET balance = balance + ?, updated = datetime('now', 'localtime')
        WHERE id = ?;
        
        UPDATE article SET usage_count = usage_count + 1
        WHERE id = ?;
		
		INSERT INTO transactions (user_id, article_id, quantity, comment, amount, deleted, created)
		VALUES (?, ?, ?, ?, ?, FALSE, datetime('now', 'localtime'));
		
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
    .fetch_one(tx).await?;

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

pub async fn add_transaction_with_recipient_tx(
    tx: &mut Transaction<'static, Sqlite>,
    mut user: model::UserEntity,
    amount: &i32,
    mut recipient: model::UserEntity,
    comment: Option<&str>,
) -> std::result::Result<model::TransactionObject, DbError> {
    let result = sqlx::query_as::<_, model::TransactionEntity>(
        "UPDATE user SET balance = balance + ?, updated = datetime('now', 'localtime')
        WHERE id = ?;
        UPDATE user SET balance = (balance + ?) * -1
		WHERE id = ?;
        
        -- sender transaction
		INSERT INTO transactions (user_id, comment, amount, deleted, created)
        VALUES (?, ?, ?, FALSE, datetime('now', 'localtime'));
        -- recipient transaction
		INSERT INTO transactions (user_id, sender_transaction_id, comment, amount, deleted, created)
		VALUES (?, last_insert_rowid(), ?, ? * -1, FALSE, datetime('now', 'localtime'));
        -- update sender transaction reference
        UPDATE transactions SET recipient_transaction_id = last_insert_rowid()
        WHERE id IN (SELECT sender_transaction_id FROM transactions WHERE id = last_insert_rowid());
		
		SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment, amount, deleted, created
        FROM transactions WHERE recipient_transaction_id = last_insert_rowid();"
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
    .fetch_one(tx).await?;

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
