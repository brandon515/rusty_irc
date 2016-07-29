use mio::{
    EventLoop,
    EventSet,
    Handler,
    Token,
};
use mio::tcp::{
    TcpListener,
    TcpStream,
};
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(0);

struct ServerHandler{
    client_list: HashMap<Token, TcpStream>,
    socket: TcpListener,
    token_counter: u32,
}

impl ServerHandler{
    fn new(listener: TcpListener, token_start: u32) -> ServerHandler{
        ServerHandler{
            client_list: HashMap::new(),
            socket: listener,
            token_counter: token_start,
        }
    }
}

impl Handler for ServerHandler{
    type Timeout = u32;
    type Message = u32;
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events:EventSet){
        if token == SERVER_TOKEN{
            println!("HEY!");
        }
    }
}
