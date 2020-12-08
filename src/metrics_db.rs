use sqlx::{sqlite::SqliteRow, Row, Sqlite, Transaction};

use crate::{
    article_db,
    error::DbError,
    model::{
        DailyTransaction, TransactionStatsEntity, TransactionSum, UserArticles, UserTransactions,
    },
};

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

pub async fn user_article_stats(
    tx: &mut Transaction<'static, Sqlite>,
    user_id: &i32,
) -> Result<Vec<UserArticles>, DbError> {
    let article_entries: Vec<(i32, i32, i32)> = sqlx::query_as(
        "SELECT COUNT(a.id) as count, SUM(t.amount) * -1 as amount, a.id
		FROM transactions AS t
		INNER JOIN article AS a ON a.id = t.article_id
		WHERE t.user_id = ?
		GROUP BY a.id
		ORDER BY COUNT(a.id) DESC",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut result: Vec<UserArticles> = vec![];
    for (count, amount, aid) in article_entries {
        let article = article_db::get_article_or_error_tx(tx, aid).await?;
        result.push(UserArticles {
            count,
            amount,
            article,
        });
    }

    Ok(result)
}

pub async fn user_transaction_stats(
    tx: &mut Transaction<'static, Sqlite>,
    user_id: &i32,
) -> Result<UserTransactions, DbError> {
    let tx_count: i32 =
        sqlx::query_scalar("SELECT COUNT(id) FROM transactions WHERE user_id = ? AND NOT deleted")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

    let incoming = user_transaction_stat(tx, user_id, true).await?;
    let outgoing = user_transaction_stat(tx, user_id, false).await?;

    Ok(UserTransactions {
        count: tx_count,
        incoming,
        outgoing,
    })
}

pub async fn user_transaction_stat(
    tx: &mut Transaction<'static, Sqlite>,
    user_id: &i32,
    incoming: bool,
) -> Result<TransactionStatsEntity, DbError> {
    let tx_row = match incoming {
        true => "sender_transaction_id",
        false => "recipient_transaction_id",
    };

    let stats: Option<TransactionStatsEntity> = sqlx::query_as(&*format!(
        "SELECT COUNT(id) AS count, SUM(amount) as amount
		FROM transactions
		WHERE user_id = ? AND {} IS NOT NULL
		GROUP BY user_id",
        tx_row
    ))
    .bind(user_id)
    .fetch_optional(tx)
    .await?;

    return Ok(stats.unwrap_or_else(|| TransactionStatsEntity {
        count: 0,
        amount: 0,
    }));
}
