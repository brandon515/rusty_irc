use ini::Ini;
pub enum Level{
    ERR,
    INFO,
    DEBUG,
}


pub fn log(err_level: Level, err_str: &str){
    let conf_file = match Ini::load_from_file("config.ini"){
        Ok(conf) => {
            conf
        },
        Err(error) => {
            println!("Logging is broken!\nReason: {:?}", error);
            return;
        },
    };
    let log_section = match conf_file.section(Some("log".to_owned())){
        Some(x) => {
            x
        },
        None => {
            println!("Logging is broken!\nReason: There is no log section in config.ini");
            return;
        }
    };
    let mut lvl_str = String::new();
    match err_level{
        Level::ERR => {
            match log_section.get("error"){
                Some(val) => {
                    if val == "true"{
                        lvl_str = "[ERROR]".to_string();
                    }else{
                        return;
                    }
                },
                None => {
                    return;
                }
            };
        },
        Level::INFO => {
            match log_section.get("info"){
                Some(val) => {
                    if val == "true"{
                        lvl_str = "[INFO]".to_string();
                    }else{
                        return;
                    }
                },
                None => {
                    return;
                }
            };
        },
        Level::DEBUG => {
            match log_section.get("debug"){
                Some(val) => {
                    if val == "true"{
                        lvl_str = "[DEBUG]".to_string();
                    }else{
                        return;
                    }
                },
                None => {
                    return;
                }
            };
        },
    };
    let time_str = "PLSHLDER";
    println!("{} {}: {}", lvl_str, time_str, err_str);
}
