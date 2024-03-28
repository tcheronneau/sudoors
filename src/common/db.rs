use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use anyhow::Context;
use log::{debug, info};

use crate::common::models::{NewSudo, Sudo};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_sudo(conn: &mut SqliteConnection, username: &str, hostnames: Option<Vec<String>>) -> anyhow::Result<usize> {
    use crate::common::schema::sudos;

    let string_hostnames = match hostnames {
        Some(hosts) => hosts.join(";"),
        None => "".to_string(),
    };

    let existing_sudo = get_sudo(conn, username)?;
    debug!("Existing sudo: {:?}", existing_sudo.len());
    if existing_sudo.len() > 0 {
        let delete_num = delete_sudo(conn, username)?;
        info!("Deleted {} records", delete_num);
    }
    let new_sudo = NewSudo { username, hostnames: string_hostnames.as_str() };

    Ok(diesel::insert_into(sudos::table)
        .values(&new_sudo)
        .execute(conn)
        .context("Failed to insert data in database")?
    )
}

pub fn delete_sudo(conn: &mut SqliteConnection, user_name: &str) -> anyhow::Result<usize> {
    use crate::common::schema::sudos::dsl::*;
    Ok(diesel::delete(sudos.filter(username.eq(user_name))).execute(conn)?)
}

pub fn get_sudo(conn: &mut SqliteConnection, user_name: &str) -> anyhow::Result<Vec<Sudo>> {
    use crate::common::schema::sudos::dsl::*;
    Ok(sudos.filter(username.eq(user_name))
        .load::<Sudo>(conn)?)
}
pub fn get_sudo_for_hostname(conn: &mut SqliteConnection, host_name: &str) -> anyhow::Result<Vec<Sudo>> {
    use crate::common::schema::sudos::dsl::*;
    let sudo_list = sudos.load::<Sudo>(conn)?;
    Ok(sudo_list.into_iter().filter(|s| s.hostnames.contains(&host_name.to_string())).collect())
}
