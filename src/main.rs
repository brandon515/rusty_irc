extern crate rusty_irc;
extern crate mio;
use mio::{
    EventLoop,
};
use rusty_irc::network_interface::{
    handler,
};

fn main() {
    let mut server_loop = EventLoop::new().unwrap();
    let mut client_handler = handler::ServerHandler::new(1234);
    handler::register_server_loop(&client_handler, &mut server_loop);
    server_loop.run(&mut client_handler).unwrap();
}
