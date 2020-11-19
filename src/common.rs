use std::{borrow::Cow, convert::Infallible, env, sync::Arc};

use regex::Regex;
use sqlx::{types::chrono::Local, SqlitePool};
use warp::Filter;

use crate::settings;

pub fn env_or(key: &str, default: &str) -> String {
    return env::var(key).ok().unwrap_or(default.to_string());
}

pub fn with_db(
    db_pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

pub fn with_settings(
    settings: settings::StrichlisteSetting,
) -> impl Filter<Extract = (Arc<settings::StrichlisteSetting>,), Error = Infallible> + Clone {
    let sw = Arc::new(settings);
    // TODO: figure out how to use reference
    warp::any().map(move || sw.clone())
}

pub fn sanitize_control_chars(input: &str) -> Cow<str> {
    let re = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
    return re.replace_all(input, "");
}

pub fn assert_email(input: &str) -> Result<(), &str> {
    let re = Regex::new(r#"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$"#)
        .unwrap();
    return match re.is_match(input) {
        true => Ok(()),
        false => Err("Value is not an email address."),
    };
}

pub fn cur_datetime_str() -> String {
    return Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
}
