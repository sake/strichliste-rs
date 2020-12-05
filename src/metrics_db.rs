use sqlx::{sqlite::SqliteRow, Row, Sqlite, Transaction};

use crate::{
    error::DbError,
    model::{DailyTransaction, TransactionSum},
};

// pub async fn system_metrics(db: &SqlitePool, date_begin: i32) -> Result<Vec<DailyTransaction>, DbError> {
// 	let mut tx = db.begin().await?;

// 	let metrics: Vec<DailyTransaction> = sqlx::query("
// 		SELECT
// 		DATE(created, 'localtime') as createDate
// 		COUNT(id) as countTransactions
// 		SUM((CASE WHEN amount >= 0 THEN 1 ELSE 0 END)) as countCharged
// 		SUM((CASE WHEN amount <  0 THEN 1 ELSE 0 END)) as countSpent
// 		COUNT(DISTINCT user) as distinctUsers
// 		SUM(amount) as amountSum
// 		SUM((CASE WHEN amount >= 0 THEN amount ELSE 0 END)) as amountCharged
// 		SUM((CASE WHEN amount <  0 THEN amount ELSE 0 END)) as amountSpent
// 		FROM transactions
// 		WHERE created = ?
// 		GROUP BY createDate
// 		ORDER BY createDate"
// 	).bind(date_begin)
// 	.try_map(|row: SqliteRow| {
// 		Ok(DailyTransaction {
// 			date: row.try_get("createDate")?,
// 			transactions: row.try_get("countTransactions")?,
// 			distinct_users: row.try_get("distinctUsers")?,
// 			balance: row.try_get("amountSum")?,
// 			charged: TransactionSum {
// 				amount: row.try_get("amountCharged")?,
// 				transactions: row.try_get("countCharged")?,
// 			},
// 			spent: TransactionSum {
// 				amount: row.try_get("amountSpent")?,
// 				transactions: row.try_get("countSpent")?,
// 			},
// 		})
// 	})
// 	.fetch_all(&mut tx).await?;

// 	Ok(metrics)
// }

pub async fn system_balance(tx: &mut Transaction<'static, Sqlite>) -> Result<i32, DbError> {
    let balance: i32 = sqlx::query_scalar("SELECT SUM(balance) FROM user WHERE NOT disabled")
        .fetch_one(tx)
        .await?;
    Ok(balance)
}

pub async fn num_transactions(tx: &mut Transaction<'static, Sqlite>) -> Result<i32, DbError> {
    let num_tx: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
        .fetch_one(tx)
        .await?;
    Ok(num_tx)
}

pub async fn num_users(tx: &mut Transaction<'static, Sqlite>) -> Result<i32, DbError> {
    let num_users: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM user")
        .fetch_one(tx)
        .await?;
    Ok(num_users)
}

pub async fn transactions_per_day(
    tx: &mut Transaction<'static, Sqlite>,
    date_begin: &str,
) -> Result<Vec<DailyTransaction>, DbError> {
    let metrics: Vec<DailyTransaction> = sqlx::query(
        "
		SELECT
		DATE(created, 'localtime') AS createDate,
		COUNT(id) AS countTransactions,
		SUM((CASE WHEN amount >= 0 THEN 1 ELSE 0 END)) AS countCharged,
		SUM((CASE WHEN amount <  0 THEN 1 ELSE 0 END)) AS countSpent,
		COUNT(DISTINCT user_id) AS distinctUsers,
		SUM(amount) AS amountSum,
		SUM((CASE WHEN amount >= 0 THEN amount ELSE 0 END)) AS amountCharged,
		SUM((CASE WHEN amount <  0 THEN amount ELSE 0 END)) AS amountSpent
		FROM transactions
		WHERE created > ?
		GROUP BY createDate
		ORDER BY createDate",
    )
    .bind(date_begin)
    .try_map(|row: SqliteRow| {
        Ok(DailyTransaction {
            date: row.try_get("createDate")?,
            transactions: row.try_get("countTransactions")?,
            distinct_users: row.try_get("distinctUsers")?,
            balance: row.try_get("amountSum")?,
            charged: TransactionSum {
                amount: row.try_get("amountCharged")?,
                transactions: row.try_get("countCharged")?,
            },
            spent: TransactionSum {
                amount: row.try_get("amountSpent")?,
                transactions: row.try_get("countSpent")?,
            },
        })
    })
    .fetch_all(tx)
    .await?;

    Ok(metrics)
}
