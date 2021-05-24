use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};
use log::error;

use super::requests;
use crate::{ib::Ib, n26::N26, saltedge::SaltEdge};

// TODO: Surely we can do something here with import_transaction.get_id()?
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
        .route("/check/n26", web::get().to(requests::check::<N26>))
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
        .route("/check/ing", web::get().to(requests::check::<SaltEdge>))
        .route(
            "/existing/ib",
            web::get().to(requests::get_existing_transactions::<Ib>),
        )
        .route(
            "/generated/ib",
            web::get().to(requests::get_generated_transactions::<Ib>),
        )
        .route(
            "/unmatched/ib",
            web::get().to(requests::get_unmatched_transactions::<Ib>),
        )
        .route(
            "/write/ib",
            web::post().to(requests::write_generated_transactions::<Ib>),
        )
        .route("/check/ib", web::get().to(requests::check::<Ib>))
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            error!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}
