#[macro_use]
extern crate lazy_static;

extern crate clap;
use clap::{App as Capp, Arg};

use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

pub mod server;
use server::MyWebSocket;

pub mod filter;
use filter::Filter;

pub mod error;
use error::MyError;

/// do websocket handshake and start `MyWebSocket` actor
fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("HttpRequest {:?}", r);
    // first lets create our actor
    let actor = MyWebSocket::new();

    // Checks if the peer connection is from a web browser
    // if it is, we accept the connection immediately
    if r.headers().contains_key("user-agent") {
        let res = ws::start(actor, &r, stream);
        println!(
            "a web agent - allowing a connection {:?}",
            res.as_ref().unwrap()
        );
        return res;
    }

    // It is a connection from a rust client
    match r.peer_addr() {
        Some(socket) => {
            // gets the client ip address
            let ip = socket.ip();
            // checks if that address is an authorized one
            if actor.filter.check_ip(&ip) {
                let res = ws::start(actor, &r, stream);
                println!("ws index INFO {:?}", res.as_ref().unwrap());
                return res;
            }

            // The address is not in our whitelist but may be, the client knows the token
            // or magic number
            // It might panic - but  think it is improbable
            if r.headers().contains_key("authorization") {
                // It is like "authorization": "Bearer 1234"
                // so out token is the second item in the returned vector
                let token = r
                    .headers()
                    .get("authorization")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split_ascii_whitespace()
                    .collect::<Vec<&str>>();
                // just check the tokens - if they are equal the client is granted
                if token[1] == &actor.filter.get_token() {
                    let res = ws::start(actor, &r, stream);
                    println!(
                        "Client knows the token - open the house to it {:?}",
                        res.as_ref().unwrap()
                    );
                    return res;
                }
            }
        }
        None => {}
    };

    Err(Error::from(MyError::NotAuthorized))
}

fn index(req: HttpRequest) -> String {
    println!("REQ: {:?}", req);
    Filter::list_clients()
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let matches = Capp::new("Server")
        .about("A websocket server")
        .arg(
            Arg::with_name("address")
                .short("i")
                .long("ip")
                .help("Set the server's ip address")
                .takes_value(true)
                .default_value("127.0.0.1")
                .required(false),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Set the server's port")
                .takes_value(true)
                .default_value("8080"),
        )
        .get_matches();

    let ip = matches.value_of("address").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("8080");
    let server_address = format!("{}:{}", ip, port);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(web::resource("/").to(index))
    })
    .bind(&server_address)?
    .run()
}
