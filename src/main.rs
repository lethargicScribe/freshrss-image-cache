use actix_web::{App, HttpServer};
use log::{debug, info};
use std::env;

mod cache;
mod handle;
mod utils;

lazy_static::lazy_static! {
    static ref ACCESS_TOKEN: String = {
        env::var("ACCESS_TOKEN").unwrap_or_else(|_| String::from(""))
    };
    static ref CACHE_DIR: String = {
        env::var("CACHE_DIR").unwrap_or_else(|_| String::from("/cache"))
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();
    info!("start server");

    info!("cache dir: {}", CACHE_DIR.to_string());
    debug!("access token: {}", ACCESS_TOKEN.to_string());

    HttpServer::new(|| {
        App::new()
            .service(handle::get_image_handle)
            .service(handle::prepare_image_handle)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
