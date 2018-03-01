extern crate socket;

use socket::{AF_INET, Socket, SOCK_DGRAM, IP_MULTICAST_TTL, IPPROTO_IP};
use std::io::{Error, ErrorKind, Result};
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
    /// Creates a new `Discovery`. Uses the default socket on the default ipv4 multicast address (239.255.255.250:1900).
    ///
    /// # Examples
    ///
    /// ```
    /// use sonos_discovery::Discovery;
    ///
    /// let discovery: Discovery = Discovery::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        let multicast_address = SocketAddr::from_str("239.255.255.250:1900")
            .map_err(|_|
                Error::new(ErrorKind::InvalidData, "Couldn't parse socket address"))?;

        Discover::with_address(multicast_address)
    }

    /// Creates a new `Discovery` with a custom multicast address.
    pub fn with_address(address: SocketAddr) -> Result<Self> {
        let socket = Discover::create_default_socket()?;
        Ok(Discover {
            multicast_addr: address,
            socket
        })
    }

    /// Create a default socket
    /// socket option: AF_INET - SOCK_DGRAM - 0 // Automatically discover the protocol (IPPROTO_UDP)
    /// socket option: IPPROTO_IP - IP_MULTICAST_TTL - 4 // UPnP 1.0 needs a TTL of 4
    fn create_default_socket() -> Result<Arc<Socket>> {
        let socket_family = AF_INET;
        let socket_level = SOCK_DGRAM;
        let protocol = 0; // auto discover
        let socket_options = vec![(IPPROTO_IP, IP_MULTICAST_TTL, 4)];

        Discover::create_socket(socket_family, socket_level, protocol, &socket_options)
    }

    fn create_socket(socket_family: i32, socket_type: i32, protocol: i32, socket_options: &[(i32, i32, i32)]) -> Result<Arc<Socket>> {
        let socket = Socket::new(socket_family, socket_type, protocol)?;
        for socket_option in socket_options {
            // TODO: Use result, allow to fail, panic or return a result?
            socket.setsockopt(socket_option.0, socket_option.1, socket_option.2)?
        }

        Ok(Arc::new(socket))
    }

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
    fn send_search(&self) -> Result<usize> {
        let player_search = br#"M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1"#;

        self.socket.sendto(player_search, 0, &self.multicast_addr)
    }

    /// Start discovering devices.
    ///
    /// # Examples
    /// In this example the search will stop if3 devices have been discovered or the default timeout (5s) is reached.
    /// This is useful if you know the amount of speakers you have and want to reduce the search time.
    ///
    /// ```
    /// use sonos_discovery::Discovery;
    ///
    /// let devices: Vec<IpAddr> = Discovery::new().unwrap().start(None, Some(3)).unwrap();
    /// ```
    pub fn start(&self, timeout: Option<u32>, device_count: Option<usize>) -> Result<Vec<IpAddr>> {
        let timeout = timeout.unwrap_or(5);
        let device_count = device_count.unwrap_or(std::u32::MAX as usize);

        let time = Instant::now();

        self.send_search()?;
        let mut devices: Vec<IpAddr> = Vec::new();
        while time.elapsed().as_secs() < u64::from(timeout) && devices.len() < device_count {
            let socket = Arc::clone(&self.socket);
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move ||
                {
                    if let Ok((__addr, _data)) = socket.recvfrom(1024, 0) {
                        // TODO: Add logging, fail on multiple send errors?
                        if sender.send((__addr, _data)).is_ok() {}
                    }
                }
            );

            // TODO: Add logging, change
            let (_addr, data) = match receiver.recv_timeout(std::time::Duration::new(0, 500_000_000)) {
                Ok((_addr, data)) => (_addr, data),
                Err(_) => continue
            };

            // Skip from_utf8_lossy
            // Due to the usual small size of `devices`, this is faster than decoding a potentially large response
            if data.is_empty() || devices.contains(&_addr.ip()) {
                println!("{:?}", &_addr.ip());
                continue
            }

            let data = String::from_utf8_lossy(&data);
            if data.contains("Sonos") {
                devices.push(_addr.ip())
            }
        }

        Ok(devices)
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
