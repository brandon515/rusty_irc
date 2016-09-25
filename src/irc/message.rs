use std::fmt;
use irc::user;

pub enum ServerMessage{
    QUIT,
    REMOVE(String, String),
    ADD(String, String),
    DISCON(String),
    SERVMSG(Message),
    CHANMSG(String, Message),
    USERMSG(String, Message),
}

pub struct Message{
    pub prefix: String,
    pub command: String,
    pub parameters: Vec<String>,
}

impl Message{
    pub fn new(raw_input: String, sender: &user::User) -> Result<Message, &'static str>{
        /*let broken_message = Message{
            prefix: "".to_string(),
            command: "".to_string(),
            parameters: Vec::new(),
            valid: false,
        };*/
        //strip out the colon if there is one
        let drain_split = match raw_input.chars().next(){
            Some(colon) => {
                if colon == ':'{
                    2
                }else{
                    1
                }
            }
            None => {
                return Err("Empty String");
            }
        };
        let mut parts: Vec<&str> = raw_input.split(' ').collect();
        //the string is only a colon or empty
        let para_iter: Vec<&str> = if parts.len() < drain_split{
            return Err("The Message is not a complete string");
        }else{
            parts.drain(drain_split..).collect()
        };
        let mut obj_parameters = Vec::new();
        let mut final_para = false;
        let mut final_para_string = String::new();
        //collect the parameters in one vec the spec say s that the final paratmenter is able to
        //have a colon and spaces in it
        'paraLoop: for para in para_iter.iter(){
            if !final_para{
                match para.chars().next(){
                    Some(ch) => {
                        if ch == ':'{
                            final_para = true;
                            final_para_string.push_str(para);
                            continue 'paraLoop;
                        }
                    },
                    None => {
                        continue 'paraLoop;
                    },
                }
                obj_parameters.push(para.to_string());
            }else{
                final_para_string.push_str(para);
            }
        }
        //push the final parameter
        obj_parameters.push(final_para_string);
        //Create the prefix if needed
        let obj_prefix = if drain_split == 2{
            match parts.get(0){
                Some(val) => {
                    val.to_string()
                },
                None => {
                    return Err("Colon but no prefix");
                },
            }
        }else{
            sender.to_string()
        };
        let obj_command = if drain_split == 2{
            match parts.get(1){
                Some(val) => {
                    val
                },
                None => {
                    return Err("No command");
                },
            }
        }else{
            match parts.get(0){
                Some(val) => {
                    val
                },
                None => {
                    return Err("No command");
                }
            }
        };
        Ok(Message{
            prefix: obj_prefix.to_string(),
            command: obj_command.to_string(),
            parameters: obj_parameters,
        })
    }
}

impl fmt::Display for Message{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let mut acc_string = String::new();
        for s in self.parameters.iter(){
            acc_string.push_str((&s).clone());
            acc_string.push_str(" ");
        }
        write!(f, "{} {} {}", self.prefix, self.command, acc_string)
    }
}
