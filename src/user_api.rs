use std::{collections::HashMap, sync::Arc};

use sqlx::SqlitePool;

use crate::{common, model, settings, user_db};

pub async fn get_users(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    query: HashMap<String, String>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let deleted: bool = query
        .get("deleted")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(false);
    let active: Option<bool> = query.get("active").map(|v| v.parse().ok()).flatten();

    let user_entities = user_db::get_users(db, settings, deleted, active);

    return match user_entities.await {
        Ok(user_entities) => {
            let result = model::UsersResp {
                count: user_entities.len(),
                users: user_entities,
            };
            Ok(Box::new(warp::reply::json(&result)))
        }
        Err(e) => {
            println!("Failed to query user table. {}", e);
            Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    };
}

pub async fn get_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    user_id: i32,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let user_entity = user_db::get_user(db, settings, user_id);

    return match user_entity.await {
        Ok(Some(user_entity)) => {
            let result = model::UserResp { user: user_entity };
            Ok(Box::new(warp::reply::json(&result)))
        }
        Ok(None) => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
        Err(e) => {
            println!("Failed to query user table. {}", e);
            Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    };
}

pub async fn find_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    query: HashMap<String, String>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let name_search = query.get("query").map(|v| v.as_str()).unwrap_or("");
    let limit: i32 = query
        .get("limit")
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or(25);

    let user_entity_result = user_db::search_user(db, settings, name_search, limit);

    return match user_entity_result.await {
        Ok(user_entity) => {
            let result = model::UsersResp {
                count: user_entity.len(),
                users: user_entity,
            };
            Ok(Box::new(warp::reply::json(&result)))
        }
        Err(e) => {
            println!("Failed to query user table. {}", e);
            Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    };
}

pub async fn add_user(
    db: SqlitePool,
    user_req: model::UserAddReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let name = user_req.name.trim();
    let name_san = common::sanitize_control_chars(name);

    let email = user_req.email;
    // check email
    match email.as_deref().map(|v| common::assert_email(v)) {
        Some(Err(_)) => {
            return Ok(Box::new(warp::http::StatusCode::FORBIDDEN));
        }
        _ => (),
    };

    let user_entity_result = user_db::create_user(db, name_san.as_ref(), email.as_deref());

    match user_entity_result.await {
        Ok(user_entity) => {
            let result = model::UserResp { user: user_entity };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        Err(e) => {
            println!("Failed to add new user. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}

pub async fn update_user(
    db: SqlitePool,
    settings: Arc<settings::StrichlisteSetting>,
    user_id: i32,
    user_req: model::UserUpdateReq,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let name = user_req.name.trim();
    let name_san = common::sanitize_control_chars(name);

    let email = user_req.email;
    // check email
    match email.as_deref().map(|v| common::assert_email(v)) {
        Some(Err(_)) => {
            return Ok(Box::new(warp::http::StatusCode::FORBIDDEN));
        }
        _ => (),
    };

    let disabled = user_req.is_disabled;

    let user_entity_result = user_db::update_user(
        db,
        settings,
        user_id,
        name_san.as_ref(),
        email.as_deref(),
        disabled,
    );

    match user_entity_result.await {
        Ok(user_entity) => {
            let result = model::UserResp { user: user_entity };
            return Ok(Box::new(warp::reply::json(&result)));
        }
        Err(e) => {
            println!("Failed to update user. {}", e);
            return Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
}
