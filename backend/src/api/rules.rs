use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};
use log::error;

use crate::{
    db::Database, import_account::ImportAccount, model::rule::Rule, n26::N26, saltedge::SaltEdge,
};

pub fn rules_routes() -> impl HttpServiceFactory {
    web::scope("/rules")
        .service(
            web::resource("/n26")
                .route(web::get().to(rules_get::<N26>))
                .route(web::post().to(rules_add::<N26>)),
        )
        .service(
            web::resource("/ing")
                .route(web::get().to(rules_get::<SaltEdge>))
                .route(web::post().to(rules_add::<SaltEdge>)),
        )
        // return json parsing errors
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            error!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}

pub fn rule_routes() -> impl HttpServiceFactory {
    web::scope("/rule")
        .service(
            web::resource("/n26")
                .route(web::get().to(get_rule))
                .route(web::delete().to(delete_rule)),
        )
        .service(
            web::resource("/ing")
                .route(web::get().to(get_rule))
                .route(web::delete().to(delete_rule)),
        )
}

async fn rules_get<T>(
    import_account: web::Data<Arc<T>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    HttpResponse::Ok().json(db.get_all_rules(Some(import_account.get_hledger_account())))
}

async fn rules_add<T>(
    import_account: web::Data<Arc<T>>,
    rule: web::Json<Rule>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    let mut rule = rule.into_inner();
    rule.import_account = import_account.get_hledger_account().to_string();
    db.create_or_update_rule(rule);
    HttpResponse::Ok().finish()
}

async fn get_rule(rule_id: web::Path<u32>, db: web::Data<Arc<Database>>) -> HttpResponse {
    match db.get_rule(*rule_id) {
        Some(r) => HttpResponse::Ok().json(r),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn delete_rule(rule_id: web::Path<u32>, db: web::Data<Arc<Database>>) -> HttpResponse {
    db.delete_rule(*rule_id);
    HttpResponse::Ok().finish()
}
