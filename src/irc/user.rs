use std::fmt;

pub struct User{
    pub name: String,
    pub host: String,
    pub nick: String,
    pub real: String,
    pub mode: u8,
}

impl User{
    pub fn new(host: String) -> User{
        User{
            name: "".to_string(),
            real: "".to_string(),
            nick: "".to_string(),
            host: host,
            mode: 255,
        }
    }
}

impl fmt::Display for User{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, ":{}!{}@{}", self.nick, self.name, self. host)
    }
}
