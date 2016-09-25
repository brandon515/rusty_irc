extern crate rusty_irc;
extern crate mio;
use mio::{
    EventLoop,
};
use rusty_irc::network_interface::{
    handler,
};
use rusty_irc::{
    logging,
};

fn main() {
    logging::log(logging::Level::INFO, "Initializing Server Loop");
    let mut server_loop = EventLoop::new().unwrap();
    logging::log(logging::Level::INFO, "Initializing Client Handler");
    let mut client_handler = handler::ServerHandler::new(1234);
    logging::log(logging::Level::INFO, "Registering Client Handler");
    handler::register_server_loop(&client_handler, &mut server_loop);
    logging::log(logging::Level::INFO, "Starting Server Loop, Ready to accept connections");
    server_loop.run(&mut client_handler).unwrap();
}
