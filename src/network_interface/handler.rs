use mio::{
    EventLoop,
    EventSet,
    Handler,
    Token,
    PollOpt,
};
use std::io::{
    Read,
};
use mio::tcp::{
    TcpListener,
    TcpStream,
    Shutdown,
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
                let start_counter = self.token_counter;
                let mut new_token = Token(self.token_counter);
                //make sure the token is not taken (usually these would be bots or serious
                //lurkers)
                while self.client_list.contains_key(&new_token){
                    if self.token_counter == usize::max_value(){
                        self.token_counter = 0;
                    }
                    //the whole client list is full, time to kick somebody
                    if self.token_counter == start_counter{
                        break;
                    }
                    self.token_counter = self.token_counter+1;
                    new_token = Token(self.token_counter);
                }
                //register a new client and tell someone if we're kicking someone out
                match self.client_list.insert(new_token.clone(), sock){
                    //reset with an old used token this shouldn't happen unless the buffer is full
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
                //new client registration
                match event_loop.register(self.client_list.get(&new_token).unwrap(), 
                                          new_token, 
                                          EventSet::readable() | EventSet::hup(), 
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
                logging::log(logging::Level::DEBUG, &(format!("The event type is: {:?}", events)));
                //client has disconnected so remove them from the list and dont reregister their
                //event
                if events.contains(EventSet::hup()){
                    let zombie_client = self.client_list.remove(&client_token).unwrap();
                    let ip = zombie_client.peer_addr().unwrap();
                    match zombie_client.shutdown(Shutdown::Both){
                        Ok(_) => {
                            logging::log(logging::Level::INFO, &(format!("Client {:?} has disconnected", ip)));
                        }
                        Err(error) => {
                            logging::log(logging::Level::ERR, &(format!("Client {:?} has not been properly shutdown", ip)));
                            logging::log(logging::Level::ERR, &(format!("Reason: {:?}", error)));
                        }
                    }
                    return;
                }
                let mut stream = self.client_list.get_mut(&client_token).unwrap();
                let ip = stream.peer_addr().unwrap();
                let mut input_bytes = Vec::new();
                //since it's a non-blocking socket an err is almost always returned so I demoted
                //this to debug and idk what to do with ok
                match stream.read_to_end(&mut input_bytes){
                    Ok(_) => {
                        logging::log(logging::Level::DEBUG, "You'll probably never see this but if you do... hello!");
                    },
                    Err(error) => {
                        logging::log(logging::Level::DEBUG, &(format!("The data from client {:?} was not able to be read (ignore if code 11)", ip)));
                        logging::log(logging::Level::DEBUG, &(format!("Reason: {:?}", error)));
                    }
                };
                match String::from_utf8(input_bytes){
                    Ok(input_string) => {
                        logging::log(logging::Level::DEBUG, &input_string);
                    },
                    Err(error) => {
                        logging::log(logging::Level::DEBUG, &(format!("input from {:?} was not utf8 complient", ip)));
                        logging::log(logging::Level::DEBUG, &(format!("Reason: {:?}", error)));
                    }
                }
                match event_loop.reregister(stream,
                                          client_token, 
                                          EventSet::readable() | EventSet::hup(), 
                                          PollOpt::edge() | PollOpt::oneshot()){
                    Ok(_)=>{
                        logging::log(logging::Level::DEBUG, &(format!("client with IP Address {:?} has been reregistered", ip)));
                    }
                    Err(error)=>{
                        logging::log(logging::Level::ERR, &(format!("Unable to reregister client {:?}", ip)));
                        logging::log(logging::Level::ERR, &(format!("Reason: {:?}", error)));
                    }
                };
            }
        }
    }
}
