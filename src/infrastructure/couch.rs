use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[derive(Debug)]
pub struct CouchError {
    pub status: u16,
    pub body: Value,
}

impl From<reqwest::Error> for CouchError {
    fn from(e: reqwest::Error) -> Self {
        CouchError {
            status: 502,
            body: json!({"error": "couchdb_no_disponible", "reason": e.to_string()}),
        }
    }
}

async fn parse(resp: reqwest::Response) -> Result<Value, CouchError> {
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap_or(json!({}));
    if status >= 400 {
        Err(CouchError { status, body })
    } else {
        Ok(body)
    }
}

pub struct CouchClient {
    http: Client,
    base_url: String,
    user: String,
    pass: String,
    pub db: String,
}

impl CouchClient {
    pub fn from_env() -> Self {
        Self {
            http: Client::new(),
            base_url: env::var("COUCHDB_URL").expect("falta COUCHDB_URL"),
            user: env::var("COUCHDB_USER").expect("falta COUCHDB_USER"),
            pass: env::var("COUCHDB_PASSWORD").expect("falta COUCHDB_PASSWORD"),
            db: env::var("COUCHDB_DB").expect("falta COUCHDB_DB"),
        }
    }

    fn url(&self, path: &str) -> String {
        return format!("{}/{}/{}", self.base_url, self.db, path);
    }

    pub async fn info(&self) -> Result<Value, CouchError> {
        let url = format!("{}/{}", self.base_url, self.db);
        let r = self.http.get(url).basic_auth(&self.user, Some(&self.pass)).send().await?;
        parse(r).await
    }

    pub async fn put_doc(&self, id: &str, doc: &Value) -> Result<Value, CouchError> {
        let r = self.http.put(self.url(id))
            .basic_auth(&self.user, Some(&self.pass))
            .json(doc).send().await?;
        parse(r).await
    }

    pub async fn get_doc(&self, id: &str) -> Result<Value, CouchError> {
        let r = self.http.get(self.url(id))
            .basic_auth(&self.user, Some(&self.pass)).send().await?;
        parse(r).await
    }

    pub async fn delete_doc(&self, id: &str, rev: &str) -> Result<Value, CouchError> {
        let r = self.http.delete(self.url(id))
            .basic_auth(&self.user, Some(&self.pass))
            .query(&[("rev", rev)]).send().await?;
        parse(r).await
    }

    pub async fn find(&self, selector: Value) -> Result<Value, CouchError> {
        let r = self.http.post(self.url("_find"))
            .basic_auth(&self.user, Some(&self.pass))
            .json(&json!({"selector": selector, "limit": 1000}))
            .send().await?;
        let body = parse(r).await?;
        Ok(body["docs"].clone())
    }

        /// Cambios desde un punto (since="0" trae todo).
    pub async fn changes(&self, since: &str) -> Result<Value, CouchError> {
        let r = self.http.get(self.url("_changes"))
            .basic_auth(&self.user, Some(&self.pass))
            .query(&[("include_docs", "true"), ("since", since)])
            .send().await?;
        parse(r).await
    }

    /// Guarda varios documentos de una sola vez.
    pub async fn bulk_docs(&self, docs: &Value) -> Result<Value, CouchError> {
        let r = self.http.post(self.url("_bulk_docs"))
            .basic_auth(&self.user, Some(&self.pass))
            .json(&json!({"docs": docs}))
            .send().await?;
        parse(r).await
    }
}