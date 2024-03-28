use std::net::SocketAddr;
use tokio::sync::watch;
use tokio::task;
use tokio::signal::unix::{signal, SignalKind};
use anyhow::Context;

mod http;
mod grpc;


pub async fn run_standalone(http_addr: &str, http_port: u16, location: &str) -> anyhow::Result<()>{
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let http_addr_ip: std::net::IpAddr = http_addr.parse().context("Failed to parse http_addr")?;
    let http_task = task::spawn(http::start_http_server(http_addr_ip, http_port, Some(location.to_string()), shutdown_rx.clone()));

    let sigint_task = task::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        sigint.recv().await;
        let _ = shutdown_tx.send(());
    });

    tokio::select! {
        _ = http_task => {},
        _ = sigint_task => {},
    }
    Ok(())
}

pub async fn run(http_addr: &str, http_port: u16, grpc_url: &str) -> anyhow::Result<()>{
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let grpc_addr: SocketAddr = grpc_url.parse().context("Failed to parse grpc_addr")?;
    let http_addr_ip: std::net::IpAddr = http_addr.parse().context("Failed to parse http_addr")?;

    let grpc_task = task::spawn(grpc::start_grpc_server(grpc_addr, shutdown_rx.clone()));

    // Spawn HTTP server task
    let http_task = task::spawn(http::start_http_server(http_addr_ip, http_port, None, shutdown_rx.clone()));

    let sigint_task = task::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        sigint.recv().await;
        let _ = shutdown_tx.send(());
    });

    tokio::select! {
        _ = http_task => {},
        _ = sigint_task => {},
        _ = grpc_task => {},
    }

    Ok(())
}

