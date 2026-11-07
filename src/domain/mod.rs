use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Entidad: Serialize + DeserializeOwned + Send + 'static {
    const TIPO: &'static str;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Propietario {
    pub nombre: String,
    pub telefono: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inmueble {
    pub propietario_id: String,
    pub tipo: String, // "casa" | "local"
    pub direccion: String,
    pub canon_mensual: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inquilino {
    pub nombre: String,
    pub telefono: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dui: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contrato {
    pub inmueble_id: String,
    pub inquilino_id: String,
    pub fecha_inicio: String, // "2026-11-01"
    pub fecha_fin: String,
    pub dia_pago: u8,
    pub canon: f64,
    pub estado: String, // "vigente" | "vencido" | "rescindido"
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pago {
    pub contrato_id: String,
    pub periodo: String, // "2026-11"
    pub fecha_vencimiento: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha_pago: Option<String>,
    pub monto: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dias_atraso: Option<i64>,
    pub estado: String, // "pendiente" | "pagado" | "atrasado"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recibo_no: Option<String>,
}

impl Entidad for Propietario { const TIPO: &'static str = "propietario"; }
impl Entidad for Inmueble    { const TIPO: &'static str = "inmueble"; }
impl Entidad for Inquilino   { const TIPO: &'static str = "inquilino"; }
impl Entidad for Contrato    { const TIPO: &'static str = "contrato"; }
impl Entidad for Pago        { const TIPO: &'static str = "pago"; }