use rodeo::*;
use rodeo::config::Config;
use std::env;

fn main() -> std::io::Result<()> {

    // |--------------------------------------------|
    // | config file creation/reading/deserializing |
    // |--------------------------------------------|

    // reads $HOME variable, returns home directory's location without a trailing slash
    // let user_home = env::var("HOME").expect("Could not get path of user's home directory!");

    let conf: Config = config::Config::new(env::args());
    println!("{:#?}", conf);

    Ok(())
}
