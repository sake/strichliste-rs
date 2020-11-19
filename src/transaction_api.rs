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
    let user = match user_db::get_user(&db, &*settings, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
        Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let transaction = match (req.amount, req.article_id, req.recipient_id) {
        // transaction with pure value
        (Some(amount), None, None) => {
            match check_limit(&*settings, &(user.balance + amount), &amount) {
                Some(r) => return Ok(r),
                None => {}
            };
            transaction_db::add_transaction_with_value(&db, user, &amount, req.comment.as_deref())
                .await
        }
        // transaction with article
        (None, Some(article_id), _) => {
            let quantity = req.quantity.unwrap_or(1);

            let article = match article_db::get_article_or_error(&db, article_id).await {
                Ok(v) => v,
                Err(_) => return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };

            let amount = article.entity.amount * quantity * -1;
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
        (Some(amount), None, Some(recipient_id)) if amount < 0 => {
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
        Err(e) => {
            println!("Failed to add transaction to DB: {}", e);
            Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
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
