use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::Arc;
use std::ops::Range;

use tokio::sync::oneshot;

use crate::relay::{RelayEntity, relaying};


pub struct RelayManager {
    allocated_ports: Arc<Mutex<HashSet<u16>>>,
}

impl RelayManager {
    pub fn new(port_range: Range<u16>) -> Self {
        let allocated_ports = port_range.collect::<HashSet<u16>>();
        Self {
            allocated_ports: Arc::new(Mutex::new(allocated_ports)),
        }
    }

    pub async fn run_relay(&self, client1: SocketAddr, client2: SocketAddr) -> Option<(u16, u16)> {
        // Get two ports for the relay
        if let Some((p0, p1)) = self.find_ports() {
            println!("Relay ports: {} -> {}", p0, p1);
            let entity1 = RelayEntity {
                port: p0,
                peer_addr: client1,
            };
            let entity2 = RelayEntity {
                port: p1,
                peer_addr: client2,
            };

            // Create a cleanup signal for the relay
            let (cleanup_tx, cleanup_rx) = oneshot::channel::<(u16, u16)>();
            let port_pool = self.allocated_ports.clone();

            // Spawn a task to wait for the cleanup signal
            let _ = tokio::spawn(async move {
                if let Ok((pc0, pc1)) = cleanup_rx.await {
                    {
                        // Put ports back into the pool
                        let mut allocated_ports = port_pool.lock().unwrap();
                        allocated_ports.insert(pc0);
                        allocated_ports.insert(pc1);
                    }
                };
            });

            println!("Launching relay: {} -> {}", p0, p1);
            // Run the relay
            let _ = tokio::spawn(async move {
                relaying(entity1, entity2, cleanup_tx).await;
            });

            println!("Returning ports: {} -> {}", p0, p1);
            return Some((p0, p1));
        } else {
            return None;
        }
    }

    fn find_ports(&self) -> Option<(u16, u16)> {
        let mut allocated_ports = self.allocated_ports.lock().unwrap();

        // Find two available ports
        let ports = allocated_ports.iter().copied().take(2).collect::<Vec<u16>>();
        if ports.len() == 2 {
            allocated_ports.remove(&ports[0]);
            allocated_ports.remove(&ports[1]);
            Some((ports[0], ports[1]))
        } else {
            None
        }
    }
}
