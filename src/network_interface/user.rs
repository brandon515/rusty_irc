use irc;
use mio::tcp::{
    TcpStream,
};

pub struct Client{
    pub socket: TcpStream,
    pub irc_user: irc::user::User,
}

impl Client{ 
    pub fn new(sock: TcpStream, ip: String) -> Client{
        Client{
            socket: sock,
            irc_user: irc::user::User::new(ip),
        }
    }

    pub fn set_nick(&mut self, nick: String){
        self.irc_user.nick = nick;
    }

    pub fn set_real(&mut self, real: String) -> Result<(),&'static str>{
        if self.irc_user.real == ""{
            self.irc_user.real = real;
            Ok(())
        }else{
            Err("Real name has already been set")
        }
    }

    pub fn set_mode(&mut self, mode: u8){
        self.irc_user.mode = mode;
    }

    pub fn set_username(&mut self, username: String) -> Result<(),&'static str>{
        if self.irc_user.name == ""{
            self.irc_user.name = username;
            Ok(())
        }else{
            Err("Username has already been set")
        }
    }
}
