use clap::{command, Parser, Subcommand};
mod agent;
mod server;
mod common;
mod cli;

#[cfg(debug_assertions)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DebugLevel;

#[cfg(debug_assertions)]
impl clap_verbosity_flag::LogLevel for DebugLevel {
    fn default() -> Option<log::Level> {
        Some(log::Level::Debug)
    }
}

#[cfg(debug_assertions)]
type DefaultLogLevel = DebugLevel;

#[cfg(not(debug_assertions))]
type DefaultLogLevel = clap_verbosity_flag::WarnLevel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct App {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<DefaultLogLevel>,
    #[clap(subcommand)]
    mode: Option<Mode>,
}

#[derive(Debug, Subcommand)]
enum Mode {
    #[command(about = "Standalone mode")]
    Standalone {
        #[clap(long, default_value = "/etc/sudoers.d/sudoors")]
        location: String,
        #[clap(long, default_value = "0.0.0.0")]
        http_listen: String,
        #[clap(long, default_value = "8000")]
        http_port: u16,
    },
    #[command(about = "Server mode")]
    Server {
        #[clap(long, default_value = "0.0.0.0")]
        http_listen: String,
        #[clap(long, default_value = "8000")]
        http_port: u16,
        #[clap(long, default_value = "0.0.0.0")]
        grpc_listen: String,
        #[clap(long, default_value = "50051")]
        grpc_port: u16,
    },
    #[command(about = "Agent mode")]
    Agent {
        #[clap(short, long)]
        server: String,
        #[clap(short, long, default_value = "50051")]
        port: u16,
        #[clap(short, long, default_value = "/etc/sudoers.d/sudoors")]
        location: String,
    },
    Create {
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        duration: u64,
        #[clap(long)]
        hostname: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let log_level = app 
        .verbose
        .log_level()
        .expect("Log level cannot be not available");

    simple_logger::init_with_level(log_level).expect("Logging successfully initialized");
    match app.mode {
        Some(Mode::Standalone { location, http_listen, http_port }) => {
            server::run_standalone(&http_listen, http_port, &location).await
        },
        Some(Mode::Server { http_listen, http_port, grpc_listen, grpc_port }) => {
            let grpc_url = format!("{}:{}", grpc_listen, grpc_port);
            server::run(&http_listen, http_port, &grpc_url).await
        },
        Some(Mode::Agent { server, port, location }) => {
            agent::run(&server, port, &location).await
        },
        Some(Mode::Create { username, duration, hostname }) => {
            let cli = cli::SudoClient::new("http://localhost:8000");
            cli.register_sudoer(&username, duration, hostname).await
        },
        None => {
            Ok(())
        }
    }
    
}

