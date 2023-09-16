use crate::db::*;
use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use deadpool_postgres::Pool;

pub type APIResult = Result<HttpResponse, Box<dyn std::error::Error>>;

#[actix_web::post("/pessoas")]
pub async fn criar_pessoa(
    pool: web::Data<Pool>,
    payload: web::Json<CriarPessoaDTO>,
) -> APIResult {
    match validate_payload(&payload) {
        Some(response) => return Ok(response),
        None => (),
    };

    let id = uuid::Uuid::new_v4().to_string();
    if insert(&pool.get().await?, &id, payload).await == 0 {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/pessoas/{id}")))
        .finish())
}

#[actix_web::get("/pessoas/{id}")]
pub async fn consultar_pessoa(
    id: web::Path<String>,
    pool: web::Data<Pool>,
) -> APIResult {
    let id = id.to_string();
    let dto = db_get_pessoa_dto(&pool.get().await?, &id).await?;
    let body = serde_json::to_string(&dto)?;
    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::get("/pessoas")]
pub async fn buscar_pessoas(
    parametros: web::Query<ParametrosBusca>,
    pool: web::Data<Pool>,
) -> APIResult {
    let t = format!("%{}%", parametros.t.to_lowercase());
    let result = db_search(&pool.get().await?, t).await?;
    let body = serde_json::to_string(&result)?;
    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::get("/contagem-pessoas")]
pub async fn contar_pessoas(pool: web::Data<Pool>) -> APIResult {
    let count: i64 = db_count(&pool.get().await?).await?;
    Ok(HttpResponse::Ok().body(count.to_string()))
}

// HELPER FUNCTIONS

fn validate_payload(payload: &CriarPessoaDTO) -> Option<HttpResponse> {
    if payload.nome.len() > 100 {
        return Some(HttpResponse::BadRequest().finish());
    }
    if payload.apelido.len() > 32 {
        return Some(HttpResponse::BadRequest().finish());
    }
    if NaiveDate::parse_from_str(&payload.nascimento, "%Y-%m-%d").is_err() {
        return Some(HttpResponse::BadRequest().finish());
    }
    if let Some(stack) = &payload.stack {
        for element in stack.clone() {
            if element.len() > 32 {
                return Some(HttpResponse::BadRequest().finish());
            }
        }
    }
    return None;
}
