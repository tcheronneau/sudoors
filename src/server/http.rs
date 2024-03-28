use rocket::{post, delete, get, http::Status,routes, State};
use rocket::serde::json::Json;
use crate::common::sudo;
use tokio::sync::watch;
use log::{debug, error};

use anyhow::Context;

pub async fn start_http_server(http_addr: std::net::IpAddr, http_port: u16, location: Option<String>, mut shutdown_rx: watch::Receiver<()>) -> anyhow::Result<()> {
    let config = rocket::config::Config {
        address: http_addr,
        port: http_port,
        ..rocket::config::Config::debug_default()
    };
    match location {
        Some(location) => {
            let sudo_run = sudo::SudoConfig {
                location: location.to_string()
            };
            match sudo_run.delete_all_sudoers() {
                Ok(result) => debug!("{}", result),
                Err(e) => error!("Error: {}", e)
            };
            rocket::custom(config)
                .mount("/", routes![create_sudo, delete_sudo])
                .manage(sudo_run)
                .launch()
                .await
                .context("Failed to launch Rocket server")?;

        },
        None => {
            rocket::custom(config)
                .mount("/", routes![register_sudo, revoke_sudo, get_sudoers])
                .launch()
                .await
                .context("Failed to launch Rocket server")?;
        } 
    }
    let _ = shutdown_rx.changed().await;
    Ok(())
}


#[post("/sudo", format="json", data = "<data>")]
pub async fn create_sudo(data: Json<sudo::Sudo>, sudo_run: &State <sudo::SudoConfig>) -> Result<String, Status> {
    let sudo = sudo::Sudo {
        username: data.username.clone(),
        duration: data.duration,
        hostnames: data.hostnames.clone(),
    };
    match sudo_run.create_temporary_sudoer(&sudo) {
        Ok(result) => Ok(result),
        Err(_) => Err(Status::InternalServerError)
    }
}
#[delete("/sudo/<user>")]
pub async fn delete_sudo(user: &str, sudo_run: &State <sudo::SudoConfig>) -> Result<String, Status> {
    match sudo_run.delete_sudoer(user) {
        Ok(result) => Ok(result),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[post("/register", format="json", data = "<data>")]
pub async fn register_sudo(data: Json<sudo::Sudo>) -> Result<String, Status> {
    let sudo = sudo::Sudo {
        username: data.username.clone(),
        duration: data.duration,
        hostnames: data.hostnames.clone(),
    };
    match sudo::register_sudoer(&sudo) {
        Ok(result) => Ok(result),
        Err(_) => Err(Status::InternalServerError)
    }
}
#[delete("/<user>")]
pub async fn revoke_sudo(user: &str) -> Result<String, Status> {
    match sudo::revoke_sudoer(user) {
        Ok(result) => Ok(result),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[get("/sudo/<user>")]
pub async fn get_sudoers(user: &str) -> Result<String, Status> {
    match sudo::get_sudoers(user) {
        Ok(sudos) => Ok(sudos.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n")),
        Err(_) => Err(Status::InternalServerError)
    }
}
