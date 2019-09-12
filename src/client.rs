//! Simple websocket client.
use std::time::Duration;
use std::{io, thread};

extern crate clap;
use clap::{App as Capp, Arg};

use actix::io::SinkWrite;
use actix::*;
use actix_codec::{AsyncRead, AsyncWrite, Framed};
use actix_http::cookie::Cookie;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    Client,
};
use futures::{
    lazy,
    stream::{SplitSink, Stream},
    Future,
};

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let matches = Capp::new("Client")
        .about("A websocket client")
        .arg(
            Arg::with_name("address")
                .short("i")
                .long("ip")
                .help("Set the server's ip address")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Set the server's port")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .help("Try to connect by using a token")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let ip = matches.value_of("address").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("8080");
    let token: String = matches.value_of("token").unwrap_or("1234").into();
    let server_address = format!("http://{}:{}/ws/", ip, port);

    let sys = actix::System::new("ws-example");

    Arbiter::spawn(lazy(move || {
        Client::new()
            .ws(&server_address)
            .bearer_auth(token)
            .connect()
            .map_err(|e| {
                println!("Error: {}", e);
            })
            .map(|(response, framed)| {
                println!("{:?}", response);
                let (sink, stream) = framed.split();
                let addr = MyClient::create(|ctx| {
                    MyClient::add_stream(stream, ctx);
                    MyClient(SinkWrite::new(sink, ctx))
                });

                // start console loop
                // here you can send instructions to the server
                // for example ADD 10.0.1.40
                // would add the 10.0.1.40 address to the whitelist
                // so a client with that ip would get a connection
                thread::spawn(move || loop {
                    let mut cmd = String::new();
                    if io::stdin().read_line(&mut cmd).is_err() {
                        println!("error");
                        return;
                    }
                    addr.do_send(ClientCommand(cmd));
                });
            })
    }));

    let _ = sys.run();
}

struct MyClient<T>(SinkWrite<SplitSink<Framed<T, Codec>>>)
where
    T: AsyncRead + AsyncWrite;

#[derive(Message)]
struct ClientCommand(String);

impl<T: 'static> Actor for MyClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl<T: 'static> MyClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.0.write(Message::Ping(String::new())).unwrap();
            act.hb(ctx);

            // client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handle stdin commands
impl<T: 'static> Handler<ClientCommand> for MyClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        self.0.write(Message::Text(msg.0)).unwrap();
    }
}

/// Handle server websocket messages
impl<T: 'static> StreamHandler<Frame, WsProtocolError> for MyClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    fn handle(&mut self, msg: Frame, _ctx: &mut Context<Self>) {
        if let Frame::Text(txt) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl<T: 'static> actix::io::WriteHandler<WsProtocolError> for MyClient<T> where
    T: AsyncRead + AsyncWrite
{
}
