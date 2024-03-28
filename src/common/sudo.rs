use std::fs;
use std::io::Write;
use tera::{Tera, Context};
use tokio::time::{Duration, sleep};
use serde::{Deserialize, Serialize};
use log::{error,info};
use std::fmt::Display;
use std::fmt;

use crate::common::db;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Sudo {
    pub username: String,
    pub duration: u64,
    #[serde(default)]
    pub hostnames: Option<Vec<String>>,
}
impl Display for Sudo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User {} has access to {} for {}", self.username, self.hostnames.as_ref().unwrap().join(","), self.duration)
    }
}

pub struct SudoConfig {
    pub location: String,
}
impl SudoConfig {
    pub fn create_sudoer_file(&self, username: String) -> anyhow::Result<String> {
        let mut context = Context::new();
        let file_path = format!("{}/{}", self.location, username);
        fs::create_dir_all(&self.location)?; 
        context.insert("username", &username);
    
        let tera = Tera::new("templates/**/*")?;
        let content = tera.render("sudoers.j2", &context)?;
        let mut file = fs::File::create(&*file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(format!("File has been created for {}", username))
    } 
    pub fn create_temporary_sudoer(&self, data: &Sudo) -> anyhow::Result<String> {
        let file_path = format!("{}/{}", self.location, data.username);
        let duration = Duration::from_secs(data.duration);
        let file_path_string = file_path.clone().to_string();
        self.create_sudoer_file(data.username.clone())?;
        tokio::spawn(async move {
            sleep(duration).await;
            if let Err(e) = fs::remove_file(file_path_string.clone()) {
                error!("Error deleting file: {}", e);
            } else {
                info!("File '{}' deleted after {} seconds", file_path_string, duration.as_secs());
            }
        });
        Ok(format!("File has been created for {}", data.username))
    } 
    pub fn delete_all_sudoers(&self) -> anyhow::Result<String> {
        fs::remove_dir_all(&self.location)?;
        Ok(format!("All files have been deleted at {}", self.location))
    }
    pub fn delete_sudoer(&self, user: &str) -> anyhow::Result<String> {
        let file_path = format!("{}/{}", self.location, user);
        fs::remove_file(&file_path)?;
        Ok(format!("File has been deleted at {}", file_path))
    }
}
    
pub fn register_sudoer(data: &Sudo) -> anyhow::Result<String> {
    let mut conn = db::establish_connection();
    let _ = db::create_sudo(&mut conn, &data.username, data.hostnames.clone())?;
    let duration = Duration::from_secs(data.duration);
    let username = data.username.clone().to_string();
    tokio::spawn(async move {
        sleep(duration).await;
        if let Err(e) = db::delete_sudo(&mut conn, &username) {
            error!("Error deleting sudo: {}", e);
        } else {
            info!("Sudo '{}' deleted after {} seconds", username, duration.as_secs());
        }
    });
    Ok(format!("User {} has been registered", data.username))
}
pub fn revoke_sudoer(username: &str) -> anyhow::Result<String> {
    let mut conn = db::establish_connection();
    match db::delete_sudo(&mut conn, username) {
        Ok(_) => Ok(format!("User {} has been revoked", username)),
        Err(e) => Err(e)
    }
}


pub fn get_sudoers(user: &str) -> anyhow::Result<Vec<Sudo>> {
    let mut conn = db::establish_connection();
    let sudos = db::get_sudo(&mut conn, user)?;
    Ok(sudos.iter().map(|s| s.to_sudo()).collect())
}

pub fn get_sudoers_for_hostname(hostname: &str) -> anyhow::Result<Vec<Sudo>> {
    let mut conn = db::establish_connection();
    Ok(db::get_sudo_for_hostname(&mut conn, hostname)?.iter().map(|s| s.to_sudo()).collect())
}

