use mio::{
    Token,
};

enum Message{
    QUIT,
    KICK(String, Token),
    JOIN(String, Token),
    DISCON(Token),
}
