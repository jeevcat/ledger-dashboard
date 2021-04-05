use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};
use log::error;

use super::{ib, requests};
use crate::{n26::N26, saltedge::SaltEdge};

pub fn transactions_routes() -> impl HttpServiceFactory {
    web::scope("/transactions")
        .route(
            "/stats",
            web::get().to(requests::get_transaction_stats::<N26>),
        )
        .route(
            "/new",
            web::post().to(requests::generate_single_transaction),
        )
        .route(
            "/existing/n26",
            web::get().to(requests::get_existing_transactions::<N26>),
        )
        .route(
            "/generated/n26",
            web::get().to(requests::get_generated_transactions::<N26>),
        )
        .route(
            "/unmatched/n26",
            web::get().to(requests::get_unmatched_transactions::<N26>),
        )
        .route(
            "/write/n26",
            web::post().to(requests::write_generated_transactions::<N26>),
        )
        .route(
            "/existing/ing",
            web::get().to(requests::get_existing_transactions::<SaltEdge>),
        )
        .route(
            "/generated/ing",
            web::get().to(requests::get_generated_transactions::<SaltEdge>),
        )
        .route(
            "/unmatched/ing",
            web::get().to(requests::get_unmatched_transactions::<SaltEdge>),
        )
        .route(
            "/write/ing",
            web::post().to(requests::write_generated_transactions::<SaltEdge>),
        )
        .route("/existing/ib", web::get().to(ib::get_existing_transactions))
        .route(
            "/unmatched/ib",
            web::get().to(ib::get_unmatched_transactions),
        )
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            error!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}