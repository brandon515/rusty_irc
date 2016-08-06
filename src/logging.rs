pub enum Level{
    ERR,
    INFO,
}

pub fn log(err_level: Level, err_str: &str){
    let err_lvl_str = match err_level{
        Level::ERR => "[ERROR]",
        Level::INFO => "[INFO]",
    };
    let time_str = "PLSHLDER";
    println!("{} {}: {}", err_lvl_str, time_str, err_str);
}
