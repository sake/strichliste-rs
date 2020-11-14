
// async fn get_transactions(
//     db: SqlitePool,
//     user_id: i32,
//     query: HashMap<String, i32>,
// ) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
//     let offset = query.get("offset").unwrap_or(&0);
//     let limit = query.get("limit").unwrap_or(&5);

//     let transactions_entities_result = sqlx::query_as::<_, model::TransactionEntity>(
//         "SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment,
//          amount, deleted, created
//          FROM transactions WHERE user_id = ? LIMIT ? OFFSET ?",
//     )
//     .bind(user_id)
//     .bind(limit)
//     .bind(offset)
//     .fetch_all(&db);

//     // let transactions_entities_result = sqlx::query(
//     //     "SELECT id, user_id, article_id, recipient_transaction_id, sender_transaction_id, quantity, comment,
//     //      amount, deleted, created
//     //      FROM transactions WHERE user_id = ? LIMIT ? OFFSET ?",
//     // )
//     // .bind(user_id)
//     // .bind(limit)
//     // .bind(offset)
//     // .map(|row: SqliteRow| TransactionEntity {
//     //     id: row.get(0),
//     //     user_id: row.get(1),
//     //     user: None,
//     //     article_id: row.get(2),
//     //     article: None,
//     //     recipient_transaction_id: row.get(3),
//     //     recipient_transaction: None,
//     //     sender_transaction_id: row.get(4),
//     //     sender_transaction: None,
//     //     quantity: row.get(5),
//     //     comment: row.get(6),
//     //     amount: row.get(7),
//     //     deleted: row.get(8),
//     //     created: row.get(9),
//     // })
//     // .fetch_all(&db);

//     return match transactions_entities_result.await {
//         Ok(result) => Ok(Box::new(warp::reply::json(&result))),
//         Err(err) => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
//     };
// }

// // async fn add_transaction(
// //     db: SqlitePool,
// //     user_id: i32,
// //     trans_req: TransactionAddReq,
// // ) -> Result<Box<dyn warp::Reply>, warp::Rejection> {

// // }
