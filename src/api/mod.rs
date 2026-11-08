mod crud;
mod pagos;
mod sync;

use crate::domain::*;
use crate::infrastructure::couch::CouchClient;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use crud::*;
use serde_json::{json, Value};
use std::sync::Arc;

fn recurso<T: Entidad>(ruta: &str) -> Router<Arc<CouchClient>> {
    Router::new()
        .route(ruta, get(listar::<T>).post(crear::<T>))
        .route(
            &format!("{}/:id", ruta),
            get(obtener::<T>).put(actualizar::<T>).delete(eliminar::<T>),
        )
}

pub fn router(couch: Arc<CouchClient>) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(recurso::<Propietario>("/propietarios"))
        .merge(recurso::<Inmueble>("/inmuebles"))
        .merge(recurso::<Inquilino>("/inquilinos"))
        .merge(recurso::<Contrato>("/contratos"))
        .merge(recurso::<Pago>("/pagos"))
        .route("/contratos/:id/generar-pagos", post(pagos::generar_pagos))
        .route("/pagos/atrasados", get(pagos::atrasados))
        .route("/pagos/:id/pagar", post(pagos::pagar))
        .route("/reportes/atraso-promedio", get(pagos::reporte_atraso))
        .route("/sync/pull", get(sync::pull))
        .route("/sync/push", post(sync::push))
        .with_state(couch)
}

async fn health(
    State(couch): State<Arc<CouchClient>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let info = couch
        .info()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.body.to_string()))?;
    Ok(Json(json!({
        "api": "ok",
        "couchdb": "ok",
        "db": info["db_name"],
        "docs": info["doc_count"]
    })))
}