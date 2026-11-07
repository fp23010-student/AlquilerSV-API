use crate::domain::Entidad;
use crate::infrastructure::couch::{CouchClient, CouchError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub type ApiResult<T> = Result<T, (StatusCode, Json<Value>)>;

pub fn err(e: CouchError) -> (StatusCode, Json<Value>) {
    (
        StatusCode::from_u16(e.status).unwrap_or(StatusCode::BAD_GATEWAY),
        Json(e.body),
    )
}

fn id_valido<T: Entidad>(id: &str) -> ApiResult<()> {
    if id.starts_with(&format!("{}:", T::TIPO)) {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, Json(json!({"error": "id_invalido"}))))
    }
}

fn a_documento<T: Entidad>(body: &T, id: &str) -> Value {
    let mut doc = serde_json::to_value(body).unwrap();
    doc["type"] = json!(T::TIPO);
    doc["_id"] = json!(id);
    doc
}

pub async fn crear<T: Entidad>(
    State(couch): State<Arc<CouchClient>>,
    Json(body): Json<T>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let id = format!("{}:{}", T::TIPO, uuid::Uuid::new_v4());
    let doc = a_documento(&body, &id);
    let res = couch.put_doc(&id, &doc).await.map_err(err)?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn listar<T: Entidad>(
    State(couch): State<Arc<CouchClient>>,
) -> ApiResult<Json<Value>> {
    let docs = couch.find(json!({"type": T::TIPO})).await.map_err(err)?;
    Ok(Json(docs))
}

pub async fn obtener<T: Entidad>(
    State(couch): State<Arc<CouchClient>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    id_valido::<T>(&id)?;
    Ok(Json(couch.get_doc(&id).await.map_err(err)?))
}

pub async fn actualizar<T: Entidad>(
    State(couch): State<Arc<CouchClient>>,
    Path(id): Path<String>,
    Json(body): Json<T>,
) -> ApiResult<Json<Value>> {
    id_valido::<T>(&id)?;
    let actual = couch.get_doc(&id).await.map_err(err)?;
    let mut doc = a_documento(&body, &id);
    doc["_rev"] = actual["_rev"].clone();
    Ok(Json(couch.put_doc(&id, &doc).await.map_err(err)?))
}

pub async fn eliminar<T: Entidad>(
    State(couch): State<Arc<CouchClient>>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    id_valido::<T>(&id)?;
    let actual = couch.get_doc(&id).await.map_err(err)?;
    let rev = actual["_rev"].as_str().unwrap_or_default().to_string();
    couch.delete_doc(&id, &rev).await.map_err(err)?;
    Ok(StatusCode::NO_CONTENT)
}