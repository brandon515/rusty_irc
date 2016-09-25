use irc::message::{
    ServerMessage,
    Message,
};
use irc::user::User;
use std::sync::mpsc::Sender;

pub fn handle(message: Message, sender: User, chan: Sender<ServerMessage>){
    //
}
