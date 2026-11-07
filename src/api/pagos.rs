use super::crud::{err, ApiResult};
use crate::application::pagos::{dias_atraso, generar_periodos};
use crate::infrastructure::couch::CouchClient;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Local, NaiveDate};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

fn fecha(v: &Value) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(v.as_str()?, "%Y-%m-%d").ok()
}

fn bad(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({"error": msg})))
}

/// POST /contratos/:id/generar-pagos
pub async fn generar_pagos(
    State(couch): State<Arc<CouchClient>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let contrato = couch.get_doc(&id).await.map_err(err)?;
    if contrato["type"] != "contrato" {
        return Err(bad("el id no es de un contrato"));
    }
    let inicio = fecha(&contrato["fechaInicio"]).ok_or_else(|| bad("fechaInicio invalida"))?;
    let fin = fecha(&contrato["fechaFin"]).ok_or_else(|| bad("fechaFin invalida"))?;
    let dia = contrato["diaPago"].as_u64().ok_or_else(|| bad("diaPago invalido"))? as u32;
    let canon = contrato["canon"].as_f64().ok_or_else(|| bad("canon invalido"))?;
    let corto = id.trim_start_matches("contrato:");

    let (mut creados, mut existentes) = (0, 0);
    for (periodo, venc) in generar_periodos(inicio, fin, dia) {
        let pago_id = format!("pago:{}:{}", periodo, corto);
        let doc = json!({
            "_id": pago_id,
            "type": "pago",
            "contratoId": id,
            "periodo": periodo,
            "fechaVencimiento": venc.format("%Y-%m-%d").to_string(),
            "monto": canon,
            "estado": "pendiente"
        });
        match couch.put_doc(&pago_id, &doc).await {
            Ok(_) => creados += 1,
            Err(e) if e.status == 409 => existentes += 1, // ya existía
            Err(e) => return Err(err(e)),
        }
    }
    Ok(Json(json!({"creados": creados, "existentes": existentes})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagarBody {
    pub fecha_pago: Option<String>,
}

/// POST /pagos/:id/pagar   (cuerpo opcional: {"fechaPago":"2026-11-08"})
pub async fn pagar(
    State(couch): State<Arc<CouchClient>>,
    Path(id): Path<String>,
    body: Option<Json<PagarBody>>,
) -> ApiResult<Json<Value>> {
    let mut doc = couch.get_doc(&id).await.map_err(err)?;
    if doc["type"] != "pago" {
        return Err(bad("el id no es de un pago"));
    }
    if doc["estado"] == "pagado" {
        return Err((StatusCode::CONFLICT, Json(json!({"error": "el pago ya fue registrado"}))));
    }
    let venc = fecha(&doc["fechaVencimiento"]).ok_or_else(|| bad("fechaVencimiento invalida"))?;
    let fecha_pago = match body.and_then(|b| b.0.fecha_pago) {
        Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|_| bad("fechaPago invalida"))?,
        None => Local::now().date_naive(),
    };

    doc["fechaPago"] = json!(fecha_pago.format("%Y-%m-%d").to_string());
    doc["diasAtraso"] = json!(dias_atraso(venc, fecha_pago));
    doc["estado"] = json!("pagado");
    doc["reciboNo"] = json!(format!("R-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase()));

    Ok(Json(couch.put_doc(&id, &doc).await.map_err(err)?))
}

/// GET /pagos/atrasados  (actualiza el estado y los días de atraso a hoy)
pub async fn atrasados(State(couch): State<Arc<CouchClient>>) -> ApiResult<Json<Value>> {
    let hoy = Local::now().date_naive();
    let docs = couch
        .find(json!({"type": "pago", "estado": {"$in": ["pendiente", "atrasado"]}}))
        .await
        .map_err(err)?;

    let mut resultado = vec![];
    for mut d in docs.as_array().cloned().unwrap_or_default() {
        let Some(venc) = fecha(&d["fechaVencimiento"]) else { continue };
        let atraso = (hoy - venc).num_days();
        if atraso <= 0 {
            continue;
        }
        if d["estado"] != "atrasado" || d["diasAtraso"] != json!(atraso) {
            d["estado"] = json!("atrasado");
            d["diasAtraso"] = json!(atraso);
            let id = d["_id"].as_str().unwrap_or_default().to_string();
            if let Ok(r) = couch.put_doc(&id, &d).await {
                d["_rev"] = r["rev"].clone();
            }
        }
        resultado.push(d);
    }
    Ok(Json(json!(resultado)))
}

/// GET /reportes/atraso-promedio  (la métrica del tema)
pub async fn reporte_atraso(State(couch): State<Arc<CouchClient>>) -> ApiResult<Json<Value>> {
    let docs = couch
        .find(json!({"type": "pago", "estado": "pagado"}))
        .await
        .map_err(err)?;
    let lista = docs.as_array().cloned().unwrap_or_default();
    let n = lista.len() as f64;
    let suma: f64 = lista.iter().filter_map(|d| d["diasAtraso"].as_f64()).sum();
    let con_atraso = lista.iter().filter(|d| d["diasAtraso"].as_f64().unwrap_or(0.0) > 0.0).count() as f64;

    Ok(Json(json!({
        "pagosRegistrados": lista.len(),
        "diasAtrasoPromedio": if n > 0.0 { suma / n } else { 0.0 },
        "porcentajeConAtraso": if n > 0.0 { con_atraso * 100.0 / n } else { 0.0 }
    })))
}