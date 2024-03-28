use sudoing::SudoingRequest;
use sudoing::sudoing_client::SudoingClient;
use gethostname::gethostname;
use log::{debug, info};

use tokio::time::{interval, Duration};
use tokio::task;
use std::sync::{Arc, Mutex};


use crate::common::sudo::SudoConfig;

pub mod sudoing {
  tonic::include_proto!("sudoing");
}

#[derive(Debug)]
pub struct State {
    users: Vec<String>,
}
#[derive(Debug)]
pub struct SudoResponse {
    pub username: String,
}

pub async fn poll_server(url: String,sudo: Arc<SudoConfig>,state: Arc<Mutex<State>>) -> anyhow::Result<()> {
    let mut client = SudoingClient::connect(url).await?;
    let hostname = gethostname().to_string_lossy().to_string();
    let request = tonic::Request::new(SudoingRequest{
        hostname,
    });

    let response = client.sudo(request).await?;
    let sudo_resp: Vec<SudoResponse> = response.into_inner().sudoers.into_iter().map(|s| {
            SudoResponse {
                username: s.username.clone(),
            }
    }).collect();
    let mut state = state.lock().unwrap();
    sudo_resp.iter().for_each(|s| {
        if !state.users.contains(&s.username) {
            sudo.create_sudoer_file(s.username.clone()).unwrap();
            debug!("User {} has been added", s.username);
        }
    });
    state.users.iter().for_each(|u| {
        if !sudo_resp.iter().any(|s| s.username == *u) {
            sudo.delete_sudoer(u).unwrap();
            debug!("User {} has been removed", u);
        }
    });

    state.users = sudo_resp.iter().map(|s| s.username.clone()).collect();
    info!("Current users: {:?}", state.users);
    Ok(())
}

pub async fn run(server: &str, port: u16, location: &str) -> anyhow::Result<()> { 
    let state = Arc::new(Mutex::new(State { users: Vec::new() }));
    let sudocfg = SudoConfig { location: location.to_string() };
    match &sudocfg.delete_all_sudoers() {
        Ok(_) => info!("All sudoers have been deleted"),
        Err(e) => info!("Error deleting all sudoers: {}", e),
    }
    let sudo = Arc::new(sudocfg);
    let mut interval = interval(Duration::from_secs(5));
    let url = format!("http://{}:{}", server, port);
    info!("Polling server at {}", url);
    loop {
        interval.tick().await;
        task::spawn(poll_server(url.clone(),sudo.clone(),state.clone()));
    }
}

