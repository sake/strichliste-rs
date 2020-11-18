use std::{collections::HashMap, sync::Arc};

use sqlx::SqlitePool;

use crate::{article_db, model, settings::StrichlisteSetting, transaction_db, user_db};

pub async fn get_transactions(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    user_id: i32,
    query: HashMap<String, i32>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let limit = query.get("limit").unwrap_or(&5);
    let offset = query.get("offset").unwrap_or(&0);

    let transactions =
        transaction_db::get_transactions(&db, &settings, &user_id, limit, offset).await;
    let num_transactions = transaction_db::num_active(&db, Some(user_id)).await;

    match (transactions, num_transactions) {
        (Ok(t), Ok(num_t)) => {
            let result = model::TransactionsResp {
                transactions: t,
                count: num_t as usize,
            };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        (Err(e), _) => {
            println!("Failed to query transactions table. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
        (_, Err(e)) => {
            println!("Failed to calculate number of users transactions. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

pub async fn add_transaction(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    user_id: i32,
    req: model::TransactionAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if (req.recipient_id.is_some() || req.article_id.is_some()) && req.amount > 0 {
        // Amount can't be positive when sending money or buying an article
        return Ok(Box::new(warp::http::StatusCode::BAD_REQUEST));
    }

    let user = match user_db::get_user(&db, &*settings, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
        Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let transaction = match (req.article_id, req.quantity, req.recipient_id) {
        // transaction with pure value
        (None, None, None) => {
            let amount = req.amount;
            match check_limit(&*settings, &(user.balance + amount), &amount) {
                Some(r) => return Ok(r),
                None => {}
            };
            transaction_db::add_transaction_with_value(&db, user, &amount, req.comment.as_deref())
                .await
        }
        // transaction with article
        (Some(article_id), Some(quantity), _) => {
            let article = match article_db::get_article_or_error(&db, article_id).await {
                Ok(v) => v,
                Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };

            let amount = req.amount * quantity * -1;
            match check_limit(&*settings, &(user.balance + amount), &amount) {
                Some(r) => return Ok(r),
                None => {}
            };

            transaction_db::add_transaction_with_article(
                &db,
                user,
                &quantity,
                &amount,
                article,
                req.comment.as_deref(),
            )
            .await
        }
        // transaction with recipient
        (None, None, Some(recipient_id)) => {
            let amount = req.amount;
            let recipient = match user_db::get_user(&db, &*settings, &recipient_id).await {
                Ok(Some(v)) => v,
                Ok(None) => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
                Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };

            // checking the payee is sufficient
            match check_limit(&*settings, &(user.balance + amount), &amount) {
                Some(r) => return Ok(r),
                None => {}
            };

            transaction_db::add_transaction_with_recipient(
                &db,
                user,
                &amount,
                recipient,
                req.comment.as_deref(),
            )
            .await
        }
        _ => return Ok(Box::new(warp::http::StatusCode::BAD_REQUEST)),
    };

    return match transaction {
        Ok(result) => Ok(Box::new(warp::reply::json(&result))),
        Err(_) => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
    };
}

fn check_limit(
    settings: &StrichlisteSetting,
    new_balance: &i32,
    amount: &i32,
) -> Option<Box<dyn warp::Reply>> {
    if &settings.account.boundary.lower > new_balance
        || new_balance > &settings.account.boundary.upper
    {
        return Some(Box::new(warp::http::StatusCode::BAD_REQUEST));
    } else if &settings.payment.boundary.lower > amount || amount > &settings.payment.boundary.upper
    {
        return Some(Box::new(warp::http::StatusCode::BAD_REQUEST));
    } else {
        return None;
    }
}
