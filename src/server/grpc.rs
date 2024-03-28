use tonic::{Request, Response, Status as TonicStatus};
use sudoing::{SudoingRequest, SudoingResponse, sudoing_server::{Sudoing, SudoingServer}};
use anyhow::Context;
use tokio::sync::watch;

use crate::common;

pub mod sudoing {
  tonic::include_proto!("sudoing");
}

#[derive(Debug, Default)]
pub struct SudoingService {}

#[tonic::async_trait]
impl Sudoing for SudoingService {
  async fn sudo(&self, request: Request<SudoingRequest>) -> Result<Response<SudoingResponse>, TonicStatus> {
    let r = request.into_inner();
    let sudos = common::sudo::get_sudoers_for_hostname(&r.hostname).expect("Error getting sudoers"); 
    let sudoers: Vec<sudoing::Sudoer> = sudos.iter().map(|s| {
        sudoing::Sudoer {
            username: s.username.clone(), 
        }
    }).collect();
    Ok(Response::new(sudoing::SudoingResponse {
        sudoers
    }))

  }
}


pub async fn start_grpc_server(addr: std::net::SocketAddr, mut shutdown_rx: watch::Receiver<()>) -> anyhow::Result<()> {
    let sudorpc = SudoingService::default();

    Ok(tonic::transport::Server::builder()
        .add_service(SudoingServer::new(sudorpc))
        .serve_with_shutdown(addr, async {
            // Wait for shutdown signal
            let _ = shutdown_rx.changed().await;
        })
        .await
        .context("gRPC server failed")?)
}

