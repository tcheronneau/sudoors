use crate::common::sudo;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug)]
pub struct SudoClient {
    url: String,
    client: Client,
}
impl SudoClient {
    pub fn new(url: &str) -> Self {
        SudoClient {
            url: url.to_string(),
            client: Client::new(),
        }
    }
    pub async fn register_sudoer(&self, username: &str, duration: u64, hostnames: Vec<String>) -> anyhow::Result<()> {
        let data = sudo::Sudo {
            username: username.to_string(),
            duration,
            hostnames: Some(hostnames),
        };
        let url = format!("{}/sudo", self.url);
        let body = serde_json::to_string(&data)?;
        self.client.post(&url)
            .body(body)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        Ok(())
    }

}
