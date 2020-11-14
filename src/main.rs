use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use warp::Filter;

use crate::common::{with_db, with_settings};

mod common;
mod db;
mod model;
mod settings;
mod settings_api;
mod user_api;
mod user_db;

const SETTINGS_FILE_ENV: &str = "SETTINGS_FILE";
const SETTINGS_FILE_DEFAULT: &str = "/etc/strichliste.yaml";

const BIND_ADDR_ENV: &str = "BIND_ADDRESS";
const BIND_ADDR_DEFAULT: &str = "[::]:3030";

const DB_FILE_ENV: &str = "DB_FILE";
const DB_FILE_DEFAULT: &str = "/var/lib/strichliste/strichliste.sqlite";

#[tokio::main]
async fn main() {
    let settings = match settings::load_settings(SETTINGS_FILE_ENV, SETTINGS_FILE_DEFAULT) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    let db_file = common::env_or(DB_FILE_ENV, DB_FILE_DEFAULT);
    let db = match db::open_db(db_file.as_str()).await {
        Ok(db) => db,
        Err(e) => panic!("{}", e),
    };
    match db::migrate_db(&db).await {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };

    // TODO: add error handling
    let addr_str = common::env_or(BIND_ADDR_ENV, BIND_ADDR_DEFAULT);
    let mut addr_iter = addr_str.to_socket_addrs().unwrap();
    let addr = addr_iter.next().unwrap();

    start_webserver(addr, db, settings).await;
}

async fn start_webserver(addr: SocketAddr, db: SqlitePool, settings: settings::StrichlisteSetting) {
    println!("Starting webserver binding to {} ...", addr);

    // see next link how to add apis
    // https://blog.logrocket.com/creating-a-rest-api-in-rust-with-warp/

    // settings API
    let settings_api = warp::get()
        .and(with_settings(settings.clone()))
        .and(warp::path!("settings"))
        .and_then(settings_api::get_settings);

    // user API
    let user_path = warp::path("user");
    let get_users = warp::get()
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and(with_settings(settings.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(user_api::get_users);
    let get_user = warp::get()
        .and(with_db(db.clone()))
        .and(with_settings(settings.clone()))
        .and(warp::path!(i32))
        .and_then(user_api::get_user);
    let find_user = warp::get()
        .and(with_db(db.clone()))
        .and(with_settings(settings.clone()))
        .and(warp::path!("search"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(user_api::find_user);
    let add_user = warp::post()
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and(warp::body::json())
        .and_then(user_api::add_user);
    let update_user = warp::post()
        .and(with_db(db.clone()))
        .and(with_settings(settings.clone()))
        .and(warp::path!(i32))
        .and(warp::body::json())
        .and_then(user_api::update_user);
    // let get_transactions_api = warp::get()
    //     .and(with_db(db.clone()))
    //     .and(warp::path!(i32 / "transaction"))
    //     .and(warp::query::<HashMap<String, i32>>())
    //     .and_then(get_transactions);
    let user_api = user_path.and(
        get_users
            .or(get_user)
            .or(find_user)
            .or(add_user)
            .or(update_user),
    );

    // let add_transaction_api = warp::post()
    //     .and(with_db(db.clone()))
    //     .and(warp::path!(i32 / "transaction"))
    //     .and(warp::body::json())
    //     .and_then(add_transaction);

    // article API
    // let article_path = warp::path!("article" / ..);
    // let get_article_api = warp::get()
    //     .and(with_db(db.clone()))
    //     .and(warp::query::<HashMap<String, String>>())
    //     .and_then(get_articles);
    // let add_article_api = warp::post()
    //     .and(with_db(db.clone()))
    //     .and(warp::body::json())
    //     .and_then(add_article);
    // let article_api = article_path.and(get_article_api.or(add_article_api));

    // bind it together
    let api = warp::path("api").and(settings_api.or(user_api) /*.or(article_api)*/);

    warp::serve(api).run(addr).await;
}
