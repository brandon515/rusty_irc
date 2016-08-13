use mio::tcp::{
    TcpStream,
};

struct Client{
    socket: TcpStream,
    irc_client: u32,
}

impl Client{
    fn new(stream: TcpStream) -> Client{
        Client{
            socket: stream,
            irc_client: 32,
        }
    }
}


