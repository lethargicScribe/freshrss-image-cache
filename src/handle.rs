use crate::cache;
use actix_files::NamedFile;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

lazy_static::lazy_static! {
    static ref ACCESS_TOKEN: String = {
        env::var("ACCESS_TOKEN").unwrap_or_else(|_| String::from(""))
    };
    static ref CACHE_DIR: String = {
        env::var("CACHE_DIR").unwrap_or_else(|_| String::from("/cache"))
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrepareJson {
    #[serde(rename = "url")]
    url: String,

    #[serde(rename = "access_token")]
    access_token: String,
}

#[post("/prepare")]
async fn prepare_image_handle(
    req_body: web::Json<PrepareJson>,
) -> Result<impl Responder, Box<dyn Error>> {
    debug!("post /prepare body is {:?}", req_body);
    if req_body.access_token != ACCESS_TOKEN.to_string() {
        return Ok(HttpResponse::BadRequest().body("invalid access_token"));
    }

    let p = cache::get_image_cache_path(&req_body.url)?;
    if cache::is_image_cache_hit(&p) {
        debug!("prepare cache hit path: {:?}", p);
        return Ok(HttpResponse::Ok().body("ok"));
    }
    cache::prepare_cache(&req_body.url).await?;
    Ok(HttpResponse::Ok().body("ok"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PicQuery {
    #[serde(rename = "url")]
    url: String,
}

#[get("/pic")]
async fn get_image_handle(
    req: HttpRequest,
    query: web::Query<PicQuery>,
) -> Result<impl Responder, Box<dyn Error>> {
    debug!("get /pic query is {:?}", query);
    let p = cache::get_image_cache_path(&query.url)?;

    if !cache::is_image_cache_hit(&p) {
        info!("cache miss path: {:?}", p);
        cache::prepare_cache(&query.url).await?;
    }
    info!("cache hit path: {:?}", p);
    let cache = NamedFile::open_async(p.clone()).await?;

    Ok(cache.into_response(&req))
}
