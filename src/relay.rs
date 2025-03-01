use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::Arc;

use tokio::sync::oneshot;
use tokio::net::UdpSocket;
use tokio::time::Instant;
use tokio::time::Duration;

// 30 mins
const CHECK_INTERVAL: Duration = Duration::from_secs(30*60);
const SELECT_TIMEOUT: Duration = Duration::from_secs(30*60);
const MAX_INACTIVITY: Duration = Duration::from_secs(30*60);

const BUFFER_SIZE: usize = 2048;

pub struct RelayEntity {
    pub port: u16,
    pub peer_addr: SocketAddr,
}

pub async fn relaying(
    entity1: RelayEntity,
    entity2: RelayEntity,
    cleanup_signal: oneshot::Sender<(u16, u16)>,
) {
    let last_updated = Arc::new(Mutex::new(Instant::now()));
    let exit_flag = Arc::new(Mutex::new(false));

    // Bind to the two UDP sockets
    let socket1 = UdpSocket::bind(("0.0.0.0", entity1.port)).await.unwrap();
    let socket2 = UdpSocket::bind(("0.0.0.0", entity2.port)).await.unwrap();

    let mut buf1 = [0u8; BUFFER_SIZE];
    let mut buf2 = [0u8; BUFFER_SIZE];

    // Spawn a background task to check the last_updated time every 10 minutes
    let last_updated_clone = Arc::clone(&last_updated);
    let exit_flag_clone = Arc::clone(&exit_flag);

    let mut entity1_last_known = entity1.peer_addr.clone();
    let mut entity2_last_known = entity2.peer_addr.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(CHECK_INTERVAL).await;
            let last_updated_time = last_updated_clone.lock().unwrap();
            if last_updated_time.elapsed() > MAX_INACTIVITY {
                *exit_flag_clone.lock().unwrap() = true;
                break;
            }
        }
    });

    // Main relay loop
    loop {
        // Exit the loop if the exit flag is set
        if *exit_flag.lock().unwrap() {
            // Stop the relay
            break;
        }

        tokio::select! {
            // Relay data from socket1 to socket2 with a timeout
            event_1to2 = tokio::time::timeout(SELECT_TIMEOUT, socket1.recv_from(&mut buf1)) => match event_1to2 {
                //Ok(Ok((len, addr))) if entity1.peer_addr == addr => {
                Ok(Ok((len, addr))) => {
                    socket2.send_to(&buf1[..len], entity2_last_known).await.unwrap();
                    entity1_last_known = addr;
                    *last_updated.lock().unwrap() = Instant::now();
                },
                //Ok(Ok((len, addr))) => {},
                Ok(Err(_)) => break,
                Err(_) => {
                    // Timeout triggered -> Reset loop
                    continue;
                }
            },

            // Relay data from socket2 to socket1 with a timeout
            event_2to1 = tokio::time::timeout(SELECT_TIMEOUT, socket2.recv_from(&mut buf2)) => match event_2to1 {
                //Ok(Ok((len, addr))) if entity1.peer_addr == addr => {
                Ok(Ok((len, addr))) => {
                    socket1.send_to(&buf2[..len], entity1_last_known).await.unwrap();
                    entity2_last_known = addr;
                    *last_updated.lock().unwrap() = Instant::now();
                },
                //Ok(Ok((len, addr))) => {},
                Ok(Err(_)) => break,
                Err(_) => {
                    // Timeout triggered -> Reset loop
                    continue;
                }
            },
        }
    }

    // Send the cleanup signal when the relay stops
    let _ = cleanup_signal.send((entity1.port, entity2.port));
    println!("Relay stopped, cleanup signal sent.");
}
