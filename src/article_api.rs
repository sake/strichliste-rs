use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::{
    article_db,
    model::{self, json_reply, JsonReply},
};

pub async fn get_articles(
    db: SqlitePool,
    query: HashMap<String, String>,
) -> Result<JsonReply<model::ArticlesResp>, warp::Rejection> {
    let ancestor: bool = query
        .get("ancestor")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(true);
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

    let articles = article_db::get_articles(&db, limit, offset, active, ancestor).await?;
    let num_articles = article_db::num_active(&db).await?;

    let result = model::ArticlesResp {
        articles,
        count: num_articles as usize,
    };

    Ok(json_reply(result))
}

pub async fn get_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<JsonReply<model::ArticleResp>, warp::Rejection> {
    let article = article_db::get_article_or_error(&db, article_id).await?;

    Ok(json_reply(model::ArticleResp { article }))
}

pub async fn add_article(
    db: SqlitePool,
    req: model::ArticleAddReq,
) -> Result<JsonReply<model::ArticleResp>, warp::Rejection> {
    let name = req.name.trim();
    let barcode = req
        .barcode
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    let article = article_db::add_article(&db, name, barcode.as_deref(), req.amount).await?;

    Ok(json_reply(model::ArticleResp { article }))
}

pub async fn update_article(
    db: SqlitePool,
    precursor_id: i32,
    req: model::ArticleAddReq,
) -> Result<JsonReply<model::ArticleResp>, warp::Rejection> {
    let name = req.name.trim();
    let barcode = req
        .barcode
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    let article =
        article_db::update_article(&db, precursor_id, name, barcode.as_deref(), req.amount).await?;

    Ok(json_reply(model::ArticleResp { article }))
}

pub async fn delete_article(
    db: SqlitePool,
    article_id: i32,
) -> Result<JsonReply<model::ArticleResp>, warp::Rejection> {
    let article = article_db::delete_article(&db, article_id).await?;

    Ok(json_reply(model::ArticleResp { article }))
}
