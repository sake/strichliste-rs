use std::sync::Arc;

use sqlx::{sqlite::SqlitePool, Result};

use crate::settings::StrichlisteSetting;
use crate::{model, settings};

pub async fn get_users(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    disabled: bool,
    active: Option<bool>,
) -> Result<Vec<model::UserEntity>> {
    // seconds until user is counted as inactive
    let stale_period = settings::get_stale_period(settings.as_ref());

    let (query_active, query_not_active) = match active {
        // query either or
        Some(v) => (v, !v),
        // query both
        None => (true, true),
    };

    let user_entities_result = sqlx::query_as::<_, model::UserEntity>(
        "SELECT id, name, email, balance, disabled, 
         CASE WHEN updated NOTNULL THEN (strftime('%s','now') - strftime('%s',updated)) < ? ELSE FALSE END as active,
         created, updated 
         FROM user WHERE disabled IS ? AND (active IS ? OR active IS NOT ?)
         ORDER BY name",
    )
    .bind(stale_period)
    .bind(disabled)
    .bind(query_active)
    .bind(query_not_active)
    .fetch_all(&db).await;

    return user_entities_result;
}

pub async fn get_user(
    db: &SqlitePool,
    settings: &StrichlisteSetting,
    user_id: i32,
) -> Result<Option<model::UserEntity>> {
    let stale_period = settings::get_stale_period(settings);

    let user_entity_result = sqlx::query_as::<_, model::UserEntity>(
        "SELECT id, name, email, balance, disabled, 
         CASE WHEN updated NOTNULL THEN (strftime('%s','now') - strftime('%s',updated)) < ? ELSE FALSE END as active,
         created, updated 
         FROM user WHERE id = ?",
	)
	.bind(stale_period)
    .bind(user_id)
    .fetch_optional(db).await;

    return user_entity_result;
}

pub async fn search_user(
    db: SqlitePool,
    settings: Arc<StrichlisteSetting>,
    name_search: &str,
    limit: i32,
) -> Result<Vec<model::UserEntity>> {
    let search = format!("%{}%", name_search);

    let stale_period = settings::get_stale_period(settings.as_ref());

    let user_entity_result = sqlx::query_as::<_, model::UserEntity>(
        "SELECT id, name, email, balance, disabled, 
         CASE WHEN updated NOTNULL THEN (strftime('%s','now') - strftime('%s',updated)) < ? ELSE FALSE END as active,
         created, updated 
		 FROM user WHERE disabled IS FALSE AND name LIKE ?
		 ORDER BY name LIMIT ?",
    )
	.bind(stale_period)
	.bind(search)
	.bind(limit)
    .fetch_all(&db).await;

    return user_entity_result;
}

pub async fn create_user(
    db: SqlitePool,
    name: &str,
    email: Option<&str>,
) -> Result<model::UserEntity> {
    let user_entity_result = sqlx::query_as::<_, model::UserEntity>(
        "INSERT INTO user (name, email, balance, disabled, created)
		 VALUES(?, ?, 0, FALSE, datetime('now'));
		 
		 SELECT id, name, email, balance, disabled, FALSE AS active, created, updated
		 FROM user WHERE id = last_insert_rowid();",
    )
    .bind(name)
    .bind(email)
    .fetch_one(&db)
    .await;

    return user_entity_result;
}

pub async fn update_user(
    db: SqlitePool,
    settings: &StrichlisteSetting,
    user_id: i32,
    name: &str,
    email: Option<&str>,
    disabled: bool,
) -> Result<model::UserEntity> {
    sqlx::query(
        "UPDATE user
		 SET name = ?, email = ?, disabled = ?
		 WHERE id = ?",
    )
    .bind(name)
    .bind(email)
    .bind(disabled)
    .bind(user_id)
    .execute(&db)
    .await?;

    return match get_user(&db, settings, user_id).await {
        Ok(v) => Ok(v.unwrap()),
        Err(e) => Err(e),
    };
}
