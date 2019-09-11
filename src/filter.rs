use parking_lot::RwLock;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

// Main list of whitelisted clients.
lazy_static! {
    static ref CLIENTS: RwLock<HashSet<IpAddr>> = RwLock::new(HashSet::new());
}
pub struct Filter;

impl Filter {
    pub fn add_ip(&mut self, ip: IpAddr) -> bool {
        {
            let mut clients = CLIENTS.write();
            println!("Adding  {:?} to the whitelist", ip);
            (*clients).insert(ip)
        }
    }
    pub fn remove_ip(&mut self, ip: &IpAddr) -> bool {
        {
            let mut clients = CLIENTS.write();
            println!("Removing  {:?} from the whitelist", ip);
            (*clients).remove(ip)
        }
    }
    pub fn check_ip(&self, ip: &IpAddr) -> bool {
        {
            let clients = CLIENTS.read();
            (*clients).contains(ip)
        }
    }
    pub fn list_clients() -> String {
        {
            let clients = CLIENTS.read();
            format!("Authorized clients: {:?}", *clients)
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        {
            let mut clients = CLIENTS.write();
            // We are a default authorized client
            (*clients).insert(IpAddr::from_str("127.0.0.1").unwrap());
        }
        Self
    }
}
