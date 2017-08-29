extern crate socket;

use socket::{AF_INET, Socket, SOCK_DGRAM, IP_MULTICAST_TTL, IPPROTO_IP};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
/// `Discover` type
///
/// Used for discovering sonos devices in the local network via the simple service discovery protocol (ssdp).
/// The ssd-protocol works via udp sockets. First a certain search-message is sent to the multicast address (239.255.255.250:1900).
///
/// All answer from upnp (universal plug and play) ready devices are processed and filtered ("Sonos" is in the reply).
pub struct Discover {
    /// Multicast address in the local network
    multicast_addr: SocketAddr,
    /// Socket implementation
    /// INFO: The socket type will likely change in the future due to cross platform compatability
    socket: Arc<Socket>
}

impl Discover {
    pub fn new() -> Self {
        Discover {
            multicast_addr: SocketAddr::from_str("239.255.255.250:1900").unwrap(),
            socket: Discover::create_socket()
        }
    /// Creates a new `Discovery`. Uses the default socket on the default ipv4 multicast address (239.255.255.250:1900).
    ///
    /// # Examples
    ///
    /// ```
    /// use sonos_discovery::Discovery;
    ///
    /// let discovery: Discovery = Discovery::new();
    /// ```
    /// Creates a new `Discovery` with a custom multicast address.
    /// Create a default socket
    /// socket option: AF_INET - SOCK_DGRAM - 0 // Automatically discover the protocol (IPPROTO_UDP)
    /// socket option: IPPROTO_IP - IP_MULTICAST_TTL - 4 // UPnP 1.0 needs a TTL of 4
    }

    fn create_socket() -> Arc<Socket> {
        let socket = Socket::new(AF_INET, SOCK_DGRAM, 0).unwrap();
        let _ = socket.setsockopt(IPPROTO_IP, IP_MULTICAST_TTL, 4);

        Arc::new(socket)
    }

    fn send_search(&self) {
    /// Sends the search message to the defined socket.
    /// Message can't have leading/trailing whitespaces (\s).
    ///
    /// # Message
    /// ```
    /// M-SEARCH * HTTP/1.1
    /// HOST: 239.255.255.250:1900
    /// MAN: "ssdp:discover"
    /// MX: 1
    /// ST: urn:schemas-upnp-org:device:ZonePlayer:1```
        let player_search = r#"M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1"#.as_bytes();
        let _ = self.socket.sendto(player_search, 0, &self.multicast_addr);
    }

    pub fn start(&self, timeout: Option<u32>, device_count: Option<usize>) -> HashSet<IpAddr> {
    /// Start discovering devices.
    ///
    /// # Examples
    /// In this example the search will stop if3 devices have been discovered or the default timeout (5s) is reached.
    /// This is useful if you know the amount of speakers you have and want to reduce the search time.
    ///
    /// ```
    /// use sonos_discovery::Discovery;
    ///
    /// let devices: HashSet<IpAddr> = Discovery::new().start(None, Some(3));
    /// ```
    pub fn start(&self, timeout: Option<u32>, device_count: Option<usize>) -> Result<HashSet<IpAddr>> {
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
        // There's probably a better way than a double clone
        let socket = self.socket.clone();
        let mut devices: HashSet<IpAddr> = HashSet::new();
        while time.elapsed().as_secs() < timeout as u64 && devices.len() < device_count {
            let socket = socket.clone();
            let (sender, receiver) = mpsc::channel();
            let _ = thread::spawn(move ||
                {
                    // TODO: Add logging
                    match socket.recvfrom(1024, 0) {
                        Ok((__addr, _data)) => {
                            let _ = sender.send((__addr, _data));
                            // TODO: Add logging, fail on multiple send errors?
                        }
                        Err(_) => {}
                    }
                }
            );

            // TODO: Add logging, change
            let (_addr, data) = match receiver.recv_timeout(std::time::Duration::new(0, 500000000)) {
                Ok((_addr, data)) => (_addr, data),
                Err(_) => continue
            };

            // Skip from_utf8_lossy
            // Due to the usual small size of `devices`, this is faster than decoding a potentially large response
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

/// Drop internal socket on going out of scope
impl Drop for Discover {
    fn drop(&mut self) {
        // Socket closes on drop automatically, better safe than sorry
        // Log failure for debugging
        let _ = self.socket.close();
    }
}

impl Default for Discover {
    fn default() -> Self {
        Discover::new()
    }
}
