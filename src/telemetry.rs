use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Default)]
struct Stat {
    peticiones: u64,
    errores_5xx: u64,
    total_us: u128,
    max_us: u128,
    bajo_15ms: u64,
}

#[derive(Default)]
pub struct Metricas {
    rutas: Mutex<HashMap<String, Stat>>,
}

/// Middleware: mide cuánto tarda cada petición, agrupada por ruta.
pub async fn medir(State(m): State<Arc<Metricas>>, req: Request, next: Next) -> Response {
    let ruta = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "otra".to_string());
    let clave = format!("{} {}", req.method(), ruta);

    let inicio = Instant::now();
    let resp = next.run(req).await;
    let us = inicio.elapsed().as_micros();

    let mut g = m.rutas.lock().unwrap();
    let s = g.entry(clave).or_default();
    s.peticiones += 1;
    s.total_us += us;
    if us > s.max_us {
        s.max_us = us;
    }
    if us < 15_000 {
        s.bajo_15ms += 1;
    }
    if resp.status().is_server_error() {
        s.errores_5xx += 1;
    }
    resp
}

/// GET /metrics
pub async fn reporte(State(m): State<Arc<Metricas>>) -> Json<Value> {
    let g = m.rutas.lock().unwrap();
    let mut rutas: Vec<Value> = g
        .iter()
        .map(|(k, s)| {
            let n = s.peticiones.max(1) as f64;
            json!({
                "ruta": k,
                "peticiones": s.peticiones,
                "errores5xx": s.errores_5xx,
                "latenciaMediaMs": s.total_us as f64 / n / 1000.0,
                "latenciaMaxMs": s.max_us as f64 / 1000.0,
                "porcentajeBajo15ms": s.bajo_15ms as f64 * 100.0 / n
            })
        })
        .collect();
    rutas.sort_by(|a, b| a["ruta"].as_str().cmp(&b["ruta"].as_str()));

    Json(json!({
        "memoriaRssMb": leer("VmRSS:") / 1024.0,
        "hilos": leer("Threads:") as u64,
        "rutas": rutas
    }))
}

fn leer(campo: &str) -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(campo))
                .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
        })
        .unwrap_or(0.0)
} 
