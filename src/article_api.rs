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

    let articles = article_db::get_articles(&db, limit, offset, active).await?;
    let num_articles = article_db::num_active(&db).await?;

    let result = model::ArticlesResp {
        articles,
        count: num_articles as usize,
    };
    return Ok(Box::new(warp::reply::json(&result)));
}

pub async fn get_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let article = article_db::get_article_or_error(&db, article_id).await?;
    return Ok(Box::new(warp::reply::json(&model::ArticleResp { article })));
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

    let article = article_db::add_article(&db, name, barcode.as_deref(), req.amount).await?;

    return Ok(Box::new(warp::reply::json(&model::ArticleResp { article })));
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

    let article =
        article_db::update_article(&db, precursor_id, name, barcode.as_deref(), req.amount).await?;

    return Ok(Box::new(warp::reply::json(&article)));
}

pub async fn delete_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let article = article_db::delete_article(&db, article_id).await?;

    return Ok(Box::new(warp::reply::json(&model::ArticleResp { article })));
}
