use mio::{
    EventLoop,
    EventSet,
    Handler,
    Token,
    PollOpt,
};
use mio::tcp::{
    TcpListener,
    TcpStream,
};
use std::collections::HashMap;
use logging;

const SERVER_TOKEN: Token = Token(0);

pub struct ServerHandler{
    client_list: HashMap<Token, TcpStream>,
    socket: TcpListener,
    token_counter: usize,
}

pub fn register_server_loop(client_handler: &ServerHandler, event_loop: &mut EventLoop<ServerHandler>){
        event_loop.register(&(client_handler.socket),
                            SERVER_TOKEN,
                            EventSet::readable(),
                            PollOpt::edge()).unwrap();
}

impl ServerHandler{
    pub fn new(port: u32) -> ServerHandler{
        let port_string = port.to_string();
        let address_string = "0.0.0.0:".to_string();
        let addr = (address_string+&port_string).parse().unwrap();
        let server_listener = TcpListener::bind(&addr).unwrap();
        ServerHandler{
            client_list: HashMap::new(),
            socket: server_listener,
            token_counter: 1,
        }
    }
}

impl Handler for ServerHandler{
    type Timeout = u32;
    type Message = u32;
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events:EventSet){
        match token{
            //mainly new clients are connecting
            SERVER_TOKEN => {
                //retrieve the sockket
                let sock = match self.socket.accept(){
                    Err(e) => {
                        logging::log(logging::Level::ERR, &(format!("Network error: {}", e)));
                        return;
                    },
                    Ok(None) => {
                        logging::log(logging::Level::ERR, "Network error: Stream is not ready");
                        return;
                    },
                    Ok(Some((sock, addr))) => {
                        logging::log(logging::Level::INFO, &(format!("Client {} has connected", addr)));
                        sock
                    }
                };
                let ip = sock.peer_addr().unwrap();
                let new_token = Token(self.token_counter);
                //register a new client and tell someone if we're kicking someone out
                match self.client_list.insert(new_token, sock){
                    //reset with an old used token, increment and keep trying
                    Some(old_value) => {
                        let info_string = format!("Client with the IP Address {:?} has been disconnected, buffer overflow", old_value.peer_addr());
                        logging::log(logging::Level::INFO, &info_string);
                        match event_loop.deregister(&old_value){
                            Ok(_) =>{
                                logging::log(logging::Level::INFO, "Old client has been deregistered");
                            },
                            Err(error) =>{
                                logging::log(logging::Level::ERR, "Old client was not able to be removed");
                                logging::log(logging::Level::ERR, &(format!("Reason: {:?}", error)));
                            },
                        };
                    }
                    //new token, nothing to worry about
                    None => {
                        logging::log(logging::Level::INFO, "Registering new client");
                    }
                };
                match event_loop.register(self.client_list.get(&new_token).unwrap(), 
                                          new_token, 
                                          EventSet::readable(), 
                                          PollOpt::edge() | PollOpt::oneshot()){
                    Ok(_)=>{
                        logging::log(logging::Level::INFO, &(format!("New client with IP Address {:?} has been registered", ip)));
                    }
                    Err(x)=>{
                        logging::log(logging::Level::ERR, &(format!("Unable to register new client {:?}", ip)));
                        logging::log(logging::Level::ERR, &(format!("Reason: {:?}", x)));
                    }
                };
            }
            //CLIENT CONNECTION
            client_token => {
                logging::log(logging::Level::INFO, "HELLO FROM THE EVENT_LOOP!");
                match event_loop.reregister(self.client_list.get(&new_token).unwrap(), 
                                          new_token, 
                                          EventSet::readable(), 
                                          PollOpt::edge() | PollOpt::oneshot()){
                    Ok(_)=>{
                        logging::log(logging::Level::INFO, &(format!("client with IP Address {:?} has been reregistered", ip)));
                    }
                    Err(x)=>{
                        logging::log(logging::Level::ERR, &(format!("Unable to reregister client {:?}", ip)));
                        logging::log(logging::Level::ERR, &(format!("Reason: {:?}", x)));
                    }
                };
            }
        }
    }
}
