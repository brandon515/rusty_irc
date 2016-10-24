use mio::{
    EventLoop,
    EventSet,
    Handler,
    Token,
    PollOpt,
};
use std::io::{
    Read,
    Write,
};
use std::thread;
use mio::tcp::{
    TcpListener,
    Shutdown,
};
use irc;
use std::collections::HashMap;
use logging;
use network_interface::{
    user,
};

const SERVER_TOKEN: Token = Token(0);

pub struct ServerHandler{
    client_list: HashMap<Token, user::Client>,
    channels: HashMap<String, Vec<Token>>,
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
            channels: HashMap::new(),
            socket: server_listener,
            token_counter: 1,
        }
    }

    fn retrieve_token(&self, username: &str) -> Option<Token>{
        for (tok, cli) in self.client_list.iter(){
            if cli.irc_user.name == username{
                return Some(tok.clone());
            }
        }
        None
    }

    fn send_message(&mut self, reciever: Token, msg: irc::message::Message) -> Result<(), String>{
        let mut cli = match self.client_list.get_mut(&reciever){
            Some(x) => {
                x
            },
            None => {
                return Err(format!("Tried to send data to client with Token {:?} but the client doesnt exist", reciever.clone()));
            },
        };
        //return the result of the write
        match cli.socket.write_all(msg.to_string().as_bytes()){
            Ok(_) => {
                Ok(())
            }
            Err(x) => {
                Err(format!("Failed to send message with the Error Kind: {:?}", x.kind()))
            }
        }
    }
}

impl Handler for ServerHandler{
    type Timeout = u32;
    type Message = irc::message::ServerMessage;
    
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message){
        match msg{
            irc::message::ServerMessage::QUIT =>{
                event_loop.shutdown();
            },
            irc::message::ServerMessage::REMOVE(chan, who) =>{
                let who_token = match self.retrieve_token(&who){
                    Some(x) => {
                        x
                    },
                    None => {
                        logging::log(logging::Level::ERR, &(format!("User \"{}\" has disconnected", who)));
                        return;
                    },
                };
                match self.channels.get_mut(&chan){
                    Some(x) =>{
                        //find out where this guy is
                        let place = match x.iter().position(|&comp| comp == who_token){
                            Some(pos) => {
                                pos
                            },
                            None => {
                                logging::log(logging::Level::ERR, &(format!("User \"{}\" is not in the channel named {}", who, chan)));
                                return;
                            },
                        };

                        logging::log(logging::Level::INFO,&(format!("User \"{}\" has been removed from the channel \"{}\"", who, chan)));
                        let user = x.remove(place);
                    }
                    None =>{
                        logging::log(logging::Level::ERR, &(format!("attempt to remove user \"{}\" from a non-existant channel", who)));
                    }
                };
            },
            irc::message::ServerMessage::ADD(chan, who) =>{
                //get token from username
                let who_token = match self.retrieve_token(&who){
                    Some(x) => {
                        x
                    },
                    None => {
                        logging::log(logging::Level::ERR, &(format!("User \"{}\" has disconnected", who.clone())));
                        return;
                    },
                };
                //get the vector of the channel btw entry API is ballin
                let ent_chan = self.channels.entry(chan).or_insert(Vec::new());
                ent_chan.push(who_token);
            },
            irc::message::ServerMessage::DISCON(who) => {
                let who_token = match self.retrieve_token(&who){
                    Some(x) => {
                        x
                    },
                    None => {
                        logging::log(logging::Level::ERR, &(format!("User \"{}\" has already been disconnected", who.clone())));
                        return;
                    },
                };
                //remove from all channels 
                'search: for (chan_name, user_list) in self.channels.iter_mut(){
                    let pos = match user_list.iter().position(|x| *x==who_token){
                        Some(x) => {
                            x
                        },
                        None => {
                            //haha
                            continue 'search;
                        },
                    };
                    let _ = user_list.remove(pos);
                }
                //remove user from register
                let _ = self.client_list.remove(&who_token);
            },
            irc::message::ServerMessage::CHANMSG(chan, what) => {
                let user_list = match self.channels.get(&chan){
                    Some(x) => {
                        x
                    },
                    None => {
                        logging::log(logging::Level::ERR, &(format!("channel \"{}\" does not exist", chan.clone())));
                        return;
                    },
                };
                let user_list_owned = user_list.to_owned();
                let msg_stream = event_loop.channel();
                for who in user_list_owned{
                    msg_stream.send(irc::message::ServerMessage::USERTOKENMSG(who.as_usize(), what.clone()));
                }
            },
            irc::message::ServerMessage::USERNAMEMSG(who, what) => {
                let user_token = match self.retrieve_token(&who){
                    Some(x) => {
                        x
                    },
                    None => {
                        logging::log(Logging::Level::ERR, &(format!("User with the name {} does not exist", who)));
                        return;
                    }
                };

                let msg_stream = event_loop.channel();
                msg_stream.send(irc::message::ServerMessage::USERTOKENMSG(user_token.as_usize(), what.clone()));
                match self.send_message(user_token.as_usize(), what.clone()){
                    Ok(()) => {
                        logging::log(Logging::Level::DEBUG, &(format!("Message: {}\nReciever: {}", what.clone(), who.clone())));
                    },
                    Err(x) => {
                        logging::log(Logging::Level::ERR, &(format!("{}", x)));
                        return;
                    },
                };
            },
            irc::message::ServerMessage::USERTOKENMSG(who, what) => {
                //
            },
        }
    }


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
                match self.client_list.insert(new_token.clone(), user::Client::new(sock, ip.to_string())){
                    //reset with an old used token this shouldn't happen unless the buffer is full
                    Some(old_value) => {
                        let info_string = format!("Client with the IP Address {:?} has been disconnected, buffer overflow", old_value.socket.peer_addr());
                        logging::log(logging::Level::INFO, &info_string);
                        match event_loop.deregister(&old_value.socket){
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
                match event_loop.register(&self.client_list.get(&new_token).unwrap().socket, 
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
                    let ip = zombie_client.socket.peer_addr().unwrap();
                    match zombie_client.socket.shutdown(Shutdown::Both){
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
                let ref mut stream = self.client_list.get_mut(&client_token).unwrap().socket;
                let ip = stream.peer_addr().unwrap();
                let mut input_bytes = Vec::new();
                //since it's a non-blocking socket an err is almost always returned so I demoted
                //this to debug and idk what to do with ok
                match stream.read_to_end(&mut input_bytes){
                    Ok(_) => {
                        logging::log(logging::Level::DEBUG, "You'll probably never see this but if you do... hello!");
                    },
                    Err(error) => {
                        match error.raw_os_error(){
                            Some(no) =>{
                                if no != 11{
                                    logging::log(logging::Level::ERR, &(format!("The data from client {:?} was not able to be read", ip)));
                                    logging::log(logging::Level::ERR, &(format!("Reason: {:?}", error)));
                                }
                            },
                            None =>{
                                logging::log(logging::Level::ERR, &(format!("The data from client {:?} was not able to be read", ip)));
                                logging::log(logging::Level::ERR, &(format!("Reason: {:?}", error)));
                            }
                        }
                    }
                };
                let in_str = match String::from_utf8(input_bytes){
                    Ok(input_string) => {
                        logging::log(logging::Level::DEBUG, &input_string);
                        input_string
                    },
                    Err(error) => {
                        logging::log(logging::Level::DEBUG, &(format!("input from {:?} was not utf8 complient", ip)));
                        logging::log(logging::Level::DEBUG, &(format!("Reason: {:?}", error)));
                        "".to_string()
                    }
                };
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
