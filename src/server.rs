use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::filter::Filter;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// Length of the max number of arguments in a string message
const MESSAGE_ARGS_LENGTH: usize = 2;

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// our withelist containing ip address
    /// of granted clients
    pub filter: Filter,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                // Parsing the received data from the client
                // There are two commands ADD and REMOVE plus and argument which
                // is an ip address. An example instruction is ADD 10.0.1.40
                let args = text.split_ascii_whitespace().collect::<Vec<&str>>();
                if args.len() != MESSAGE_ARGS_LENGTH {
                    ctx.text(text);
                    return;
                }
                match args[0] {
                    "ADD" => {
                        if let Ok(ip) = IpAddr::from_str(args[1]) {
                            self.add_ip(ip);
                        }
                    }
                    "REMOVE" => {
                        if let Ok(ip) = IpAddr::from_str(args[1]) {
                            self.remove_ip(ip);
                        }
                    }

                    _ => {
                        ctx.text(text);
                    }
                }
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl MyWebSocket {
    pub fn new() -> Self {
        println!("Clients {:?}", Filter::list_clients());
        Self {
            hb: Instant::now(),
            filter: Filter::default(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
    pub fn add_ip(&mut self, ip: IpAddr) -> bool {
        self.filter.add_ip(ip)
    }

    pub fn remove_ip(&mut self, ip: IpAddr) -> bool {
        self.filter.remove_ip(&ip)
    }
}
