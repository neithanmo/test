use parking_lot::RwLock;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

extern crate rand;
use rand::Rng;

// Our default token
const DEFAULT_TOKEN: u64 = 1234;

// Main list of whitelisted clients.
lazy_static! {
    static ref CLIENTS: RwLock<HashSet<IpAddr>> = RwLock::new(HashSet::new());
}

// A filter which manage the access to the whitelist of clients
// Also it generates a new token whenever a new ip is added into the list
// this token would be shown in the log so that a new user can use it to connect to the server
// even though it is not registered in the whitelist
// ( obviously i is not safe showing the token in the console)
pub struct Filter(u64);

impl Filter {
    pub fn add_ip(&mut self, ip: IpAddr) -> bool {
        {
            let mut clients = CLIENTS.write();
            println!("Adding  {:?} to the whitelist", ip);
            let mut rng = rand::thread_rng();
            self.0 = rng.gen::<u64>();
            println!("new token: {}", self.0);
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
    // returns the current connection token (magic number )
    pub fn get_token(&self) -> String {
        format!("{}", self.0)
    }
}

impl Default for Filter {
    fn default() -> Self {
        {
            let mut clients = CLIENTS.write();
            // We are a default authorized client
            (*clients).insert(IpAddr::from_str("127.0.0.1").unwrap());
        }
        println!("Default connection token: {}", DEFAULT_TOKEN);
        Self(DEFAULT_TOKEN)
    }
}
