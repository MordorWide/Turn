mod manager;
mod relay;

use std::net::SocketAddr;
use std::net::IpAddr;
use std::sync::Arc;

use warp::Filter;
use serde::{Deserialize, Serialize};

use dotenv;
use std::env;

use crate::manager::RelayManager;

#[derive(Deserialize)]
struct InputData {
    client_ip_0: String,
    client_port_0: u16,
    client_ip_1: String,
    client_port_1: u16,
}

#[derive(Serialize)]
struct Response {
    success: bool,
    relay_port_0: Option<u16>,
    relay_port_1: Option<u16>,
}

#[tokio::main]
async fn main() {
    // Load configuration
    dotenv::dotenv().ok();

    let RANGE_START = env::var("RELAY_PORT_RANGE_START").unwrap_or(
        "40000".to_string()
    ).parse::<u16>().unwrap();

    let RANGE_END = env::var("RELAY_PORT_RANGE_END").unwrap_or(
        "50000".to_string()
    ).parse::<u16>().unwrap();

    let CMD_PORT = env::var("CMD_PORT").unwrap_or(
        "8080".to_string()
    ).parse::<u16>().unwrap();

    let CMD_HOST: IpAddr = env::var("CMD_HOST").unwrap_or(
        "0.0.0.0".to_string()
    ).parse::<IpAddr>().unwrap();

    // Create a RelayManager instance
    let relay_manager = Arc::new(RelayManager::new(RANGE_START..RANGE_END));

    // POST endpoint that receives the JSON payload
    let route = warp::post()
        .and(warp::path("launch"))
        .and(warp::body::json())
        .and_then(move |data: InputData| {
            let relay_manager = Arc::clone(&relay_manager);

            async move {
                let socket_addr0 = format!("{}:{}", data.client_ip_0, data.client_port_0);
                let socket_addr1 = format!("{}:{}", data.client_ip_1, data.client_port_1);

                let socket1: SocketAddr = socket_addr0.parse().unwrap();
                let socket2: SocketAddr = socket_addr1.parse().unwrap();

                let response = if let Some((p0, p1)) = relay_manager.run_relay(
                    socket1,
                    socket2
                ).await {
                    println!("Relay launched: ({}:{}@{}) <-> ({}:{}@{})",
                        &data.client_ip_0,
                        &data.client_port_0,
                        p0,
                        &data.client_ip_1,
                        &data.client_port_1,
                        p1,
                    );
                    Response {
                        success: true,
                        relay_port_0: Some(p0),
                        relay_port_1: Some(p1),
                    }
                } else {
                    println!("Relay failed: ({}:{}) <-//-> ({}:{})",
                        &data.client_ip_0,
                        &data.client_port_0,
                        &data.client_ip_1,
                        &data.client_port_1,
                    );
                    Response {
                        success: false,
                        relay_port_0: None,
                        relay_port_1: None,
                    }
                };

                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            }
        });

    // Start the Warp server
    println!("Server running at http://{}:{} with relay range: {}-{}", CMD_HOST, CMD_PORT, RANGE_START, RANGE_END);
    warp::serve(route).run((CMD_HOST, CMD_PORT)).await;
}
