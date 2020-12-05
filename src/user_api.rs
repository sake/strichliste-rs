use std::{collections::HashMap, sync::Arc};

use model::UsersResp;
use sqlx::SqlitePool;

use crate::{
    common,
    error::DbError,
    model::{self, json_reply, JsonReply, UserResp},
    settings, user_db,
};

pub async fn get_users(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    query: HashMap<String, String>,
) -> Result<JsonReply<UsersResp>, warp::Rejection> {
    let deleted: bool = query
        .get("deleted")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(false);
    let active: Option<bool> = query.get("active").map(|v| v.parse().ok()).flatten();

    let user_entities = user_db::get_users(&db, settings, deleted, active).await?;

    let result = model::UsersResp {
        count: user_entities.len(),
        users: user_entities,
    };

    Ok(json_reply(result))
}

pub async fn get_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    user_id: i32,
) -> Result<JsonReply<UserResp>, warp::Rejection> {
    let user_entity = user_db::get_user(&db, &settings, &user_id)
        .await?
        .ok_or(DbError::EntityNotFound(format!("User")))?;

    let result = model::UserResp { user: user_entity };

    Ok(json_reply(result))
}

pub async fn find_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    query: HashMap<String, String>,
) -> Result<JsonReply<UsersResp>, warp::Rejection> {
    let name_search = query.get("query").map(|v| v.as_str()).unwrap_or("");
    let limit: i32 = query
        .get("limit")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(25);

    let user_entity = user_db::search_user(&db, settings, name_search, limit).await?;

    let result = model::UsersResp {
        count: user_entity.len(),
        users: user_entity,
    };

    Ok(json_reply(result))
}

pub async fn add_user(
    db: SqlitePool,
    user_req: model::UserAddReq,
) -> Result<JsonReply<UserResp>, warp::Rejection> {
    let name = user_req.name.trim();
    let name_san = common::sanitize_control_chars(name);

    let email = user_req.email.map(|v| v.trim().to_owned());
    // check email
    email
        .as_deref()
        .filter(|v| !v.is_empty())
        .map(|v| common::assert_email(v))
        .transpose()?;

    let user_entity = user_db::create_user(&db, name_san.as_ref(), email.as_deref()).await?;

    let result = model::UserResp { user: user_entity };

    Ok(json_reply(result))
}

pub async fn update_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    user_id: i32,
    user_req: model::UserUpdateReq,
) -> Result<JsonReply<UserResp>, warp::Rejection> {
    let name = user_req.name.trim();
    let name_san = common::sanitize_control_chars(name);

    let email = user_req.email.map(|v| v.trim().to_owned());
    // check email
    email
        .as_deref()
        .filter(|v| !v.is_empty())
        .map(|v| common::assert_email(v))
        .transpose()?;

    let disabled = user_req.is_disabled;

    let user_entity = user_db::update_user(
        &db,
        &settings,
        user_id,
        name_san.as_ref(),
        email.as_deref(),
        disabled,
    )
    .await?;

    let result = model::UserResp { user: user_entity };

    Ok(json_reply(result))
}
