use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::{article_db, model};

pub async fn get_articles(
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
    // let ancestor: bool = query
    //     .get("ancestor")
    //     .map(|v| v.parse().ok())
    //     .flatten()
    //     .unwrap_or(false);

    let articles = article_db::get_articles(&db, limit, offset, active).await;
    let num_articles = article_db::num_active(&db).await;

    match (articles, num_articles) {
        (Ok(a), Ok(num_a)) => {
            let result = model::ArticlesResp {
                articles: a,
                count: num_a as usize,
            };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        (Err(e), _) => {
            println!("Failed to query article table. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
        (_, Err(e)) => {
            println!("Failed to calculate number of active articles. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

pub async fn get_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    return match article_db::get_article(&db, Some(article_id)).await {
        Ok(Some(result)) => Ok(Box::new(warp::reply::json(&result))),
        Ok(None) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
        Err(e) => {
            println!("Failed to query article table. {}", e);
            Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    };
}

pub async fn add_article(
    db: SqlitePool,
    req: model::ArticleAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let name = req.name.trim();
    let barcode = req
        .barcode
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    let article_result = article_db::add_article(&db, name, barcode.as_deref(), req.amount);

    return match article_result.await {
        Ok(result) => Ok(Box::new(warp::reply::json(&result))),
        Err(e) => {
            println!("Failed to add new article. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

pub async fn update_article(
    db: SqlitePool,
    precursor_id: i32,
    req: model::ArticleAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let name = req.name.trim();
    let barcode = req
        .barcode
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    let article_result = article_db::update_article(&db, precursor_id, name, barcode.as_deref(), req.amount);

    return match article_result.await {
        Ok(result) => Ok(Box::new(warp::reply::json(&result))),
        Err(e) => {
            println!("Failed to update article. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

pub async fn delete_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let article_result = article_db::delete_article(&db, article_id);

    return match article_result.await {
        Ok(result) => Ok(Box::new(warp::reply::json(&result))),
        Err(e) => {
            println!("Failed to delete article. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}
