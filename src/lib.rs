extern crate socket;

use socket::{AF_INET, Socket, SOCK_DGRAM, IP_MULTICAST_TTL, IPPROTO_IP};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct Discover {
    multicast_addr: SocketAddr,
    socket: Arc<Socket>
}

impl Discover {
    pub fn new() -> Self {
        Discover {
            multicast_addr: SocketAddr::from_str("239.255.255.250:1900").unwrap(),
            socket: Discover::create_socket()
        }
    }

    fn create_socket() -> Arc<Socket> {
        let socket = Socket::new(AF_INET, SOCK_DGRAM, 0).unwrap();
        let _ = socket.setsockopt(IPPROTO_IP, IP_MULTICAST_TTL, 4);

        Arc::new(socket)
    }

    fn send_search(&self) {
        let player_search = r#"M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1"#.as_bytes();
        let _ = self.socket.sendto(player_search, 0, &self.multicast_addr);
    }

    pub fn start(&self, timeout: Option<u32>, device_count: Option<usize>) -> HashSet<IpAddr> {
        let timeout = match timeout {
            Some(value) => { value }
            None => 5
        };
        let device_count = match device_count {
            Some(value) => { value }
            None => std::u32::MAX as usize
        };

        let time = Instant::now();

        self.send_search();
        let socket = self.socket.clone();
        let mut devices: HashSet<IpAddr> = HashSet::new();
        while time.elapsed().as_secs() < timeout as u64 && devices.len() < device_count {
            let socket = socket.clone();
            let (sender, receiver) = mpsc::channel();
            let _ = thread::spawn(move ||
                {
                    match socket.recvfrom(1024, 0) {
                        Ok((__addr, _data)) => {
                            let _ = sender.send((__addr, _data));
                        }
                        Err(_) => {}
                    }
                }
            );

            let (_addr, data) = match receiver.recv_timeout(std::time::Duration::new(0, 500000000)) {
                Ok((_addr, data)) => (_addr, data),
                Err(_) => continue
            };
            if devices.contains(&_addr.ip()) || data.is_empty() {
                println!("{:?}", &_addr.ip());
                continue
            }

            let data = String::from_utf8_lossy(&data);
            if data.contains("Sonos") {
                devices.insert(_addr.ip());
            }
        }

        devices
    }
}

impl Drop for Discover {
    fn drop(&mut self) {
        // Socket closes on drop automatically, better safe than sorry
        let _ = self.socket.close();
    }
}

impl Default for Discover {
    fn default() -> Self {
        Discover::new()
    }
}
