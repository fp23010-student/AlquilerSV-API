use super::crud::{err, ApiResult};
use crate::infrastructure::couch::CouchClient;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PullQuery {
    pub since: Option<String>,
}

/// GET /sync/pull?since=<lastSeq>
pub async fn pull(
    State(couch): State<Arc<CouchClient>>,
    Query(q): Query<PullQuery>,
) -> ApiResult<Json<Value>> {
    let since = q.since.unwrap_or_else(|| "0".to_string());
    let cambios = couch.changes(&since).await.map_err(err)?;
    let docs: Vec<Value> = cambios["results"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|fila| !fila["id"].as_str().unwrap_or_default().starts_with("_design/"))
        .map(|fila| fila["doc"].clone())
        .collect();
    Ok(Json(json!({ "lastSeq": cambios["last_seq"], "docs": docs })))
}

/// POST /sync/push   cuerpo: {"docs":[...]}
pub async fn push(
    State(couch): State<Arc<CouchClient>>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let docs = body.get("docs").cloned().unwrap_or(json!([]));
    if !docs.is_array() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "docs debe ser una lista"})),
        ));
    }
    let resultado = couch.bulk_docs(&docs).await.map_err(err)?;
    Ok(Json(json!({ "resultados": resultado })))
}