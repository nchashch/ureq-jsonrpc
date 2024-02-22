use base64::{engine::general_purpose, Engine as _};
use miette::Diagnostic;
pub use ureq::{json, serde_json::Value};

#[derive(Debug, Clone)]
pub struct Client {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub id: String,
}

impl Client {
    pub fn send_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &[ureq::serde_json::Value],
    ) -> Result<Option<T>, Error> {
        let auth = format!("{}:{}", self.user, self.password);
        let resp = ureq::post(format!("http://{}:{}", self.host, self.port).as_str())
            .set("host", format!("{}:{}", self.host, self.port).as_str())
            .set("content-type", "application/json")
            .set(
                "authorization",
                format!("Basic {}", general_purpose::STANDARD.encode(auth)).as_str(),
            )
            .set("connection", "close")
            .send_json(json!({
                "jsonrpc":"2.0",
                "id": &self.id,
                "method": method,
                "params": params}
            ))?;
        let result: JsonRpcResult<T> = resp.into_json()?;
        match (result.result, result.error) {
            (Some(value), None) => Ok(Some(value)),
            (None, None) => Ok(None),
            (_, Some(error)) => Err(error.into()),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JsonRpcResult<T> {
    pub result: Option<T>,
    pub error: Option<RpcError>,
    pub id: String,
}

#[derive(thiserror::Error, Debug, Diagnostic, serde::Serialize, serde::Deserialize)]
#[error("code: {code}, message: {message}")]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    #[error("rpc error")]
    Rpc(#[from] RpcError),
    #[error("invalid json RPC response")]
    JsonRpc,
    #[error("ureq error")]
    Ureq(#[from] ureq::Error),
    #[error("failed to parse json")]
    Json(#[from] std::io::Error),
}
