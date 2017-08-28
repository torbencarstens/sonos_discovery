# Sonos discovery
Library in rust to discover sonos devices via ssdp (UPnP discovery)

# Usage

##### Cargo.toml
```toml
sonos_discovery = "0.0.1"
```

##### Rust
```rust
extern crate sonos_discovery;

use sonos_discovery::Discover;
use std::net::IpAddr;

fn main() {
    let discovery: Discover = Discover::new();
    // fn start(self, timeout: Option<u32>, device_count: Option<usize>)
    // timeout default: 5 | device_count: u32::MAX
    // Checks that {discovered_devices} < {device_count} && {elapsed_time} < {timeout}
    // Waits until 3 devices are found, or 5seconds have elapsed
    let sonos_ips: HashSet<IpAddr> = discovery.start(None, Some(3));
    for sonos_ip in sonos_ips {
        println!("{}", sonos_ip);
    }
}
```
