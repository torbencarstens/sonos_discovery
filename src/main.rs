extern crate sonos_discovery;

use sonos_discovery::Discover;

fn main() {
    let discovery = Discover::new().unwrap();
    let devices = discovery.start(None, Some(3)).unwrap();
    for device in devices {
        println!("{:?}", device)
    }
}
