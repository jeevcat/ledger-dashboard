use std::sync::{Arc, Mutex};

use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};

use crate::{db::Database, model::rule::Rule, templater::Templater};

pub fn rules_routes() -> impl HttpServiceFactory {
    web::resource("/rules")
        .route(web::get().to(rules_get))
        .route(web::post().to(rules_add))
        // return json parsing errors
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            println!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}

pub fn rule_routes() -> impl HttpServiceFactory {
    web::resource("/rule/{rule_id}")
        .route(web::get().to(get_rule))
        .route(web::delete().to(delete_rule))
}

async fn rules_get(db: web::Data<Arc<Database>>) -> HttpResponse {
    HttpResponse::Ok().json(db.get_all_rules())
}

async fn rules_add(
    item: web::Json<Rule>,
    db: web::Data<Arc<Database>>,
    templater: web::Data<Arc<Mutex<Templater<'_>>>>,
) -> HttpResponse {
    println!("model: {:?}", &item);
    let rule = item.0;
    templater.lock().unwrap().register_rule(&rule).unwrap();
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
