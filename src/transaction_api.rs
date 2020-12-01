use std::{collections::HashMap, sync::Arc};

use sqlx::SqlitePool;

use crate::{
    article_db,
    error::{ClientError, DbError},
    model,
    model::TransactionResp,
    settings::StrichlisteSetting,
    transaction_db, user_db,
};

pub async fn get_transactions(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    user_id: i32,
    query: HashMap<String, i32>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let limit = query.get("limit").unwrap_or(&5);
    let offset = query.get("offset").unwrap_or(&0);

    let transactions =
        transaction_db::get_transactions(&db, &settings, &user_id, limit, offset).await?;
    let num_transactions = transaction_db::num_active(&db, Some(user_id)).await?;

    let result = model::TransactionsResp {
        transactions,
        count: num_transactions as usize,
    };
    return Ok(Box::new(warp::reply::json(&result)));
}

pub async fn add_transaction(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    user_id: i32,
    req: model::TransactionAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let mut tx = db.begin().await.map_err(|e| -> DbError { e.into() })?;
    let user = match user_db::get_user_tx(&mut tx, &*settings, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
        Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let transaction = match (req.amount, req.article_id, req.recipient_id) {
        // transaction with pure value
        (Some(amount), None, None) => {
            check_limit(&*settings, &(user.balance + amount), &amount)?;
            let result = transaction_db::add_transaction_with_value_tx(
                &mut tx,
                user,
                &amount,
                req.comment.as_deref(),
            )
            .await?;
            // TODO: find out how to use the From trait for this more elegantly
            tx.commit().await.map_err(|e| -> DbError { e.into() })?;
            result
        }
        // transaction with article
        (None, Some(article_id), _) => {
            let quantity = req.quantity.unwrap_or(1);

            let article = match article_db::get_article_or_error(&db, article_id).await {
                Ok(v) => v,
                Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };

            let amount = article.entity.amount * quantity * -1;
            check_limit(&*settings, &(user.balance + amount), &amount)?;

            let result = transaction_db::add_transaction_with_article_tx(
                &mut tx,
                user,
                &quantity,
                &amount,
                article,
                req.comment.as_deref(),
            )
            .await?;
            tx.commit().await.map_err(|e| -> DbError { e.into() })?;
            result
        }
        // transaction with recipient
        (Some(amount), None, Some(recipient_id)) if amount < 0 => {
            let recipient = match user_db::get_user_tx(&mut tx, &*settings, &recipient_id).await {
                Ok(Some(v)) => v,
                Ok(None) => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
                Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };

            // checking the payee is sufficient
            check_limit(&*settings, &(user.balance + amount), &amount)?;

            let result = transaction_db::add_transaction_with_recipient_tx(
                &mut tx,
                user,
                &amount,
                recipient,
                req.comment.as_deref(),
            )
            .await?;
            tx.commit().await.map_err(|e| -> DbError { e.into() })?;
            result
        }
        _ => {
            return Err(ClientError::ParameterInvalid(
                "Parameters don't match any addTransaction functionality.".to_string(),
            )
            .into())
        }
    };

    Ok(Box::new(warp::reply::json(&TransactionResp { transaction })))
}

fn check_limit(
    settings: &StrichlisteSetting,
    new_balance: &i32,
    amount: &i32,
) -> Result<(), ClientError> {
    if &settings.account.boundary.lower > new_balance
        || new_balance > &settings.account.boundary.upper
    {
        return Err(ClientError::ParameterInvalid(
            "Requested balance is out of the allowed boundary.".to_string(),
        ));
    } else if &settings.payment.boundary.lower > amount || amount > &settings.payment.boundary.upper
    {
        return Err(ClientError::ParameterInvalid(
            "Requested amount is out of the allowed boundary.".to_string(),
        ));
    } else {
        return Ok(());
    }
}
