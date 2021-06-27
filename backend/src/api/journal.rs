use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use serde::Deserialize;

use crate::git;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveRequestModel {
    commit_msg: String,
}

pub fn journal_routes() -> impl HttpServiceFactory {
    web::scope("/journal")
        .route("/save", web::post().to(save_journal))
        .route("/dirty", web::get().to(get_dirty_files))
}

async fn save_journal(body: web::Json<SaveRequestModel>) -> HttpResponse {
    git::commit_and_push(&body.commit_msg).unwrap();
    HttpResponse::Ok().finish()
}

async fn get_dirty_files() -> HttpResponse {
    let files = git::get_dirty_files().unwrap();
    HttpResponse::Ok().json(files)
}
