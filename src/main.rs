#![allow(clippy::let_unit_value)]
use std::net;
use std::str::FromStr;

use actix::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::FramedRead;

mod chat_utils;
mod codec;
mod server;
mod session;

use codec::ChatCodec;
use server::ChatServer;
use session::ChatSession;

/// Define TCP server that will accept incoming TCP connection and create
/// chat actors.
struct Server {
    chat: Addr<ChatServer>,
}

/// Make actor from `Server`
impl Actor for Server {
    /// Every actor has to provide execution `Context` in which it can run.
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for Server {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        // For each incoming connection we create `ChatSession` actor
        // with out chat server address.
        let server = self.chat.clone();
        ChatSession::create(move |ctx| {
            let (r, w) = tokio::io::split(msg.0);
            ChatSession::add_stream(FramedRead::new(r, ChatCodec), ctx);
            ChatSession::new(server, actix::io::FramedWrite::new(w, ChatCodec, ctx))
        });
    }
}

#[actix::main]
async fn main() {
    // Start chat server actor
    let server = ChatServer::default().start();

    // Create server listener
    let addr = net::SocketAddr::from_str("127.0.0.1:8080").unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();

    // create a stream with the help of stream macro.
    let stream = async_stream::stream! {
        // accept incoming tcp stream and remote addr and return them as a stream
        while let Ok((st, addr)) = listener.accept().await {
            yield TcpConnect(st, addr);
        }
    };

    // Our chat server `Server` is an actor, first we need to start it
    // and then add stream on incoming tcp connections to it.
    // TcpListener::incoming() returns stream of the (TcpStream, net::SocketAddr)
    // items So to be able to handle this events `Server` actor has to implement
    // stream handler `StreamHandler<(TcpStream, net::SocketAddr), io::Error>`
    Server::create(move |ctx| {
        ctx.add_message_stream(stream);
        Server { chat: server }
    });

    println!("Running chat server on 127.0.0.1:8080");

    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}
