use std::{collections::HashMap, ops::Sub};

use sqlx::SqlitePool;

use crate::{
    error::DbError,
    metrics_db,
    model::SystemMetrics,
    model::{json_reply, JsonReply},
};

pub async fn get_sys_metrics(
    db: SqlitePool,
    query: HashMap<String, String>,
) -> Result<JsonReply<SystemMetrics>, warp::Rejection> {
    let days: u32 = query
        .get("days")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(30);
    let date_begin = chrono::Local::now()
        .sub(chrono::Duration::days(days as i64))
        .format("%F")
        .to_string();

    let mut tx = db.begin().await.map_err(|e| -> DbError { e.into() })?;

    let balance = metrics_db::system_balance(&mut tx).await?;
    let transaction_count = metrics_db::num_transactions(&mut tx).await?;
    let user_count = metrics_db::num_users(&mut tx).await?;
    let articles = vec![];
    let days = metrics_db::transactions_per_day(&mut tx, &*date_begin).await?;

    tx.commit().await.map_err(|e| -> DbError { e.into() })?;

    let metrics = SystemMetrics {
        balance,
        transaction_count,
        user_count,
        articles,
        days,
    };

    Ok(json_reply(metrics))
}

// pub async fn get_user_metrics(
//     db: SqlitePool,
// 	settings: Arc<settings::StrichlisteSetting>,
// 	uid: i32,
// ) -> Result<Box<dyn warp::Reply>, warp::Rejection> {

// }
