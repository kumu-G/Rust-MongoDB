use chrono::prelude::*;
use db::DB;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{Filter, Rejection};
type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
use bson::serde_helpers::chrono_datetime_as_bson_datetime;

mod db;
mod error;
mod handler;

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub name: String,
    pub author: String,
    pub num_pages: usize,
    // https://docs.rs/bson/latest/bson/serde_helpers/chrono_datetime_as_bson_datetime/#:~:text=Module%20bson%20%3A%3A%20serde_helpers%20%3A%3A%20chrono_datetime_as_bson_datetime%20source%20%C2%B7,crate%3A%3ADateTime%20and%20deserialize%20a%20chrono%3A%3ADateTime%20from%20a%20crate%3A%3ADateTime.
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub added_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = DB::init().await?;
    let book = warp::path("book");

    let book_routes = book
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handler::create_book_handler)
        .or(book
            .and(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and(with_db(db.clone()))
            .and_then(handler::edit_book_handler))
        .or(book
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_db(db.clone()))
            .and_then(handler::delete_book_handler))
        .or(book
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(handler::books_list_handler));

    let routes = book_routes.recover(error::handler_rejection);
    println!("Started on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
