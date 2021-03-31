use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};
use log::{error, info};
use serde::Deserialize;

use crate::{db::Database, model::rule::Rule};

#[derive(Deserialize)]
struct RulesFilter {
    import_account: Option<String>,
}

pub fn rules_routes() -> impl HttpServiceFactory {
    web::resource("/rules")
        .route(web::get().to(rules_get))
        .route(web::post().to(rules_add))
        // return json parsing errors
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            error!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}

pub fn rule_routes() -> impl HttpServiceFactory {
    web::resource("/rule/{rule_id}")
        .route(web::get().to(get_rule))
        .route(web::delete().to(delete_rule))
}

async fn rules_get(query: web::Query<RulesFilter>, db: web::Data<Arc<Database>>) -> HttpResponse {
    HttpResponse::Ok().json(db.get_all_rules(query.import_account.as_deref()))
}

async fn rules_add(rule: web::Json<Rule>, db: web::Data<Arc<Database>>) -> HttpResponse {
    db.create_or_update_rule(rule.into_inner());
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
