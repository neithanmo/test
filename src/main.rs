#[macro_use]
extern crate lazy_static;

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
    match r.peer_addr() {
        Some(socket) => {
            let ip = socket.ip();
            let actor = MyWebSocket::new();
            if actor.filter.check_ip(&ip) {
                let res = ws::start(MyWebSocket::new(), &r, stream);
                println!("ws index INFO {:?}", res.as_ref().unwrap());
                return res;
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

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            // static files
            //.service(fs::Files::new("/", "static/").index_file("index.html"))
            .service(web::resource("/").to(index))
    })
    // start http server on 127.0.0.1:8080
    .bind("127.0.0.1:8080")?
    .run()
}
