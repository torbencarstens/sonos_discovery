extern crate socket;

use socket::{AF_INET, Socket, SOCK_DGRAM, IP_MULTICAST_TTL, IPPROTO_IP};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Instant;

fn main() {
    Discover::new().start(None, Some(3))
}

struct Discover {
    pub devices: Vec<String>,
    multicast: SocketAddr,
    socket: Socket
}

impl Discover {
    pub fn new() -> Self {
        Discover {
            devices: Vec::new(),
            multicast: SocketAddr::from_str("239.255.255.250:1900").unwrap(),
            socket: Discover::create_socket()
        }
    }

    fn create_socket() -> Socket {
        let socket = Socket::new(AF_INET, SOCK_DGRAM, 0).unwrap();
        let _ = socket.setsockopt(IPPROTO_IP, IP_MULTICAST_TTL, 4);

        socket
    }

    fn send_search(&self) {
        let player_search = r#"M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1"#.as_bytes();
        let _ = self.socket.sendto(player_search, 0, &self.multicast);
    }

    pub fn start(self, timeout: Option<u32>, device_count: Option<u32>) {
        let timeout = match timeout {
            Some(value) => { value }
            None => 5
        };
        let device_count = match device_count {
            Some(value) => { value }
            None => std::u32::MAX
        };

        self.send_search();
        let mut count = 0;
        let time = Instant::now();

        while time.elapsed().as_secs() < timeout as u64 && count < device_count {
            let (_addr, data) = self.socket.recvfrom(1024, 0).unwrap();
            let data = String::from_utf8_lossy(&data);
            if data.contains("Sonos") {
                println!("{}", data);
                count += 1;
            }
        }

        let _ = self.socket.close();
    }
}
