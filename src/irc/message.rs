use std::mem;

pub struct Message{
    prefix: String,
    command: String,
    parameters: Vec<String>,
    valid: bool,
}

impl Message{
    fn new(raw_input: String) -> Message{
        let broken_message = Message{
            prefix: "".to_string(),
            command: "".to_string(),
            parameters: Vec::new(),
            valid: false,
        };
        let drain_split = match raw_input.chars().next(){
            Some(colon) => {
                if colon == ':'{
                    2
                }else{
                    1
                }
            }
            None => {
                return broken_message;
            }
        };
        let mut parts: Vec<&str> = raw_input.split(' ').collect();
        let para_iter: Vec<&str> = if parts.len() <= drain_split{
            return broken_message;
        }else{
            parts.drain(drain_split..).collect()
        };
        let mut obj_parameters = Vec::new();
        let mut final_para = false;
        let mut final_para_string = String::new();
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
                        return broken_message;
                    },
                }
                obj_parameters.push(para.to_string());
            }else{
                final_para_string.push_str(para);
            }
        }
        obj_parameters.push(final_para_string);
        let obj_prefix = if drain_split == 2{
            match parts.get(0){
                Some(val) => {
                    val
                },
                None => {
                    return broken_message;
                },
            }
        }else{
            ""
        };
        let obj_command = if drain_split == 2{
            match parts.get(1){
                Some(val) => {
                    val
                },
                None => {
                    return broken_message;
                },
            }
        }else{
            match parts.get(0){
                Some(val) => {
                    val
                },
                None => {
                    return broken_message;
                }
            }
        };
        Message{
            prefix: obj_prefix.to_string(),
            command: obj_command.to_string(),
            parameters: obj_parameters,
            valid: true,
        }
    }
}
