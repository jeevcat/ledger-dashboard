use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};
use log::error;

use super::requests;
use crate::{ib::Ib, n26::N26, saltedge::SaltEdge};

pub fn transactions_routes() -> impl HttpServiceFactory {
    web::scope("/transactions")
        .service(
            web::scope("/new")
                .route(
                    "/n26",
                    web::post().to(requests::generate_single_transaction::<N26>),
                )
                .route(
                    "/ing",
                    web::post().to(requests::generate_single_transaction::<SaltEdge>),
                )
                .route(
                    "/ib",
                    web::post().to(requests::generate_single_transaction::<Ib>),
                ),
        )
        .service(
            web::scope("/existing")
                .route(
                    "/n26",
                    web::get().to(requests::get_existing_transactions::<N26>),
                )
                .route(
                    "/ing",
                    web::get().to(requests::get_existing_transactions::<SaltEdge>),
                )
                .route(
                    "/ib",
                    web::get().to(requests::get_existing_transactions::<Ib>),
                ),
        )
        .service(
            web::scope("/generated")
                .route(
                    "/n26",
                    web::get().to(requests::get_generated_transactions::<N26>),
                )
                .route(
                    "/ing",
                    web::get().to(requests::get_generated_transactions::<SaltEdge>),
                )
                .route(
                    "/ib",
                    web::get().to(requests::get_generated_transactions::<Ib>),
                ),
        )
        .service(
            web::scope("/unmatched")
                .route(
                    "/n26",
                    web::get().to(requests::get_unmatched_transactions::<N26>),
                )
                .route(
                    "/ing",
                    web::get().to(requests::get_unmatched_transactions::<SaltEdge>),
                )
                .route(
                    "/ib",
                    web::get().to(requests::get_unmatched_transactions::<Ib>),
                ),
        )
        .service(
            web::scope("/write")
                .route(
                    "/n26",
                    web::post().to(requests::write_generated_transactions::<N26>),
                )
                .route(
                    "/ing",
                    web::post().to(requests::write_generated_transactions::<SaltEdge>),
                )
                .route(
                    "/ib",
                    web::post().to(requests::write_generated_transactions::<Ib>),
                ),
        )
        .service(
            web::scope("/check")
                .route("/n26", web::get().to(requests::check::<N26>))
                .route("/ing", web::get().to(requests::check::<SaltEdge>))
                .route("/ib", web::get().to(requests::check::<Ib>)),
        )
        .route(
            "/stats",
            web::get().to(requests::get_transaction_stats::<N26>),
        )
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            error!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}
