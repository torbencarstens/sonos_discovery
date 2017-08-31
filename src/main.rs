extern crate sonos_discovery;

use sonos_discovery::Discover;
use std::time::Instant;

fn main() {
    let start_time = Instant::now();

    let discovery = Discover::new().unwrap();
    let ips = discovery.start(None, Some(3)).unwrap();
    for ip in ips {
        println!("{:?}", ip)
    }

    println!("\nTime: {:?}", start_time.elapsed())
}
