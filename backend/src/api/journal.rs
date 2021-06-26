use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::git;

pub fn journal_routes() -> impl HttpServiceFactory {
    web::scope("/journal").route("/save", web::post().to(save_journal))
}

async fn save_journal() -> HttpResponse {
    git::commit_and_push("test commit message").unwrap();
    HttpResponse::Ok().finish()
}
