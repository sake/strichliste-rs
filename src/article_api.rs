
async fn add_article(
    db: SqlitePool,
    user_req: model::UserAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // let user_entity_result = sqlx::query_as::<_, UserEntity>(
    //     "INSERT INTO user (name, balance, disabled, created)
    //      VALUES('f00bar', 0, FALSE, datetime('now'));
    //      SELECT id, name, email, balance, disabled, FALSE AS active, created, updated
    //      FROM user WHERE id = last_insert_rowid();",
    // )
    // .bind(user_req.name)
    // .fetch_one(&db);

    // match user_entity_result.await {
    //     Ok(user_entities) => {
    //         let result = UsersResponse {
    //             users: vec![user_entities],
    //         };
    //         return Ok(Box::new(warp::reply::json(&result)));
    //     }
    //     Err(e) => {
    //         println!("Failed to add new article. {}", e);
    //         return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
    //     }
    // };

    return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
}

async fn get_articles(
    db: SqlitePool,
    query: HashMap<String, String>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let active: bool = query
        .get("active")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(true);
    let limit: i32 = query
        .get("limit")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(999);
    let offset: i32 = query
        .get("offset")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(0);
    let ancestor: bool = query
        .get("ancestor")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(false);

    let article_entities_result = sqlx::query_as::<_, model::ArticleEntity> (
        "SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
         FROM article WHERE active IS ? LIMIT ? OFFSET ?",
    )
    .bind(active)
    .bind(limit)
    .bind(offset)
    .fetch_all(&db);

    match article_entities_result.await {
        Ok(mut article_entities) => {
            for next in article_entities.iter_mut() {
                let r = add_precursor_articles(db.clone(), next).await;
                match r {
                    Err(e) => {
                        println!("Failed to add precursor tables. {}", e);
                        return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
                    }
                    Ok(_) => (),
                };
            }

            let result = model::ArticlesResp {
                count: article_entities.len(),
                articles: article_entities,
            };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        Err(e) => {
            println!("Failed to query article table. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

async fn add_precursor_articles(
    db: SqlitePool,
    root: &mut model::ArticleEntity,
) -> Result<(), sqlx::Error> {
    let mut next_precursor_id: Option<i32> = root.precursor_id.clone();
    let mut precursors: Vec<Box<model::ArticleEntity>> = Vec::new();

    // retreive everything from the DB
    while let Some(pid) = next_precursor_id {
        let article_result = sqlx::query(
            "SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
                FROM article WHERE id = ?",
        )
        .bind(pid)
        .map(|row: SqliteRow| model::ArticleEntity {
            id: row.get(0),
            precursor_id: row.get(1),
            name: row.get(2),
            barcode: row.get(3),
            amount: row.get(4),
            active: row.get(5),
            created: row.get(6),
            usage_count: row.get(7),
        })
        .fetch_optional(&db);

        match article_result.await? {
            Some(v) => {
                next_precursor_id = v.precursor_id;
                precursors.push(Box::new(v));
            }
            None => {
                break;
            }
        }
    }

    // set references
    if precursors.len() > 0 {
        precursors.reverse();
        let mut last = precursors.pop().unwrap();
        while let Some(mut next) = precursors.pop() {
            next.precursor = Some(last);
            last = next;
        }
        root.precursor = Some(last);
    }

    return Ok(());
}
