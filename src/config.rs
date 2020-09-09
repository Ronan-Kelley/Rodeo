use std::env;
use std::path;

#[derive(Clone, Debug)]
pub struct Config {
    config_path: path::PathBuf,
    primary_command: String,
    primary_command_args: Vec<String>,
    // output information regarding program operation to stdout (I.E., what files are being
    // moved/copied/etc
    // default value: true
    verbose: bool,
    // prefer files located in their destination locations on the disk when syncing
    // default value: false
    prefer_disk: bool,
    // prefer files located in their origin location in the local repository when syncing
    // default value: false
    prefer_repo: bool,
}

impl Config {
    pub fn default() -> Self {
        // get user home directory with no trailing slash
        let mut user_home = env::var("HOME").unwrap_or_default();
        match &user_home[user_home.len() - 1..] {
            "/" => user_home = user_home[..user_home.len() - 1].to_owned(),
            _ => (),
        }

        Config {
            // default path for the config file is ~/.config/rodeo/rodeo.toml
            config_path: path::PathBuf::from(format!("{}/.config/rodeo/rodeo.toml", user_home)),
            primary_command: String::new(),
            primary_command_args: Vec::new(),
            verbose: true,
            prefer_disk: false,
            prefer_repo: false,
        }
    }

    pub fn new(args: env::Args) -> Self {
        // start with a base containing the default values
        let mut base_cfg = Config::default();

        // flag variable to keep track of whether or not the next arg is a part of the last parsed
        // argument - I.E., '-c ~/.rodeo.toml' is one argument, not two.
        let mut last_arg_multipart: bool = false;
        let mut last_arg: String = String::new();

        /*
         * parse every arg given the following strategy:
         *      args beginning with a single '-' are multi-part and as such must set the
         *          'last_arg_multipart' flag, and will generally set a string variable.
         *      args beginning with '--' are single part, and set a boolean flag.
         *      args not beginning with '-' or '--' are to be considered part of the command being
         *          passed to Rodeo. The first such arg will be considered the command, with all
         *          subsequent args being considered arguments for the command itself.
         */
        for cur_arg in args.skip(1) {
            let mut dash_count: u8 = 0;
            
            // determine type of arg, unless the last arg was a multipart arg - in which case, the
            // argument is assumed to be an argument for the last argument itself.
            if !last_arg_multipart {
                match cur_arg.trim().get(0..2) {
                    // unwrap first 2 chars
                    Some(val) => match val {
                        // if first two chars are both dashes, set dash count as 2
                        "--" => dash_count = 2,
                        // otherwise, try to unwrap first of the two chars
                        _ => match val.chars().nth(0) {
                            // if successful, set dash count appropriately
                            Some(sub_val) => match sub_val {
                                '-' => dash_count = 1,
                                _ => dash_count = 0,
                            },
                            // if not successful, something weird is going on - skip this arg.
                            None => continue,
                        }
                    },
                    // if the arg can't be unwrapped, it will be skipped and consequently ignored.
                    // While this may be to the detriment of the kind of psychopath that uses emojis in
                    // their terminal, that's something I'm willing to live with.
                    None => continue,
                };
            } else {
                // while at the moment this could be an if statement with no else or else if, it is
                // a match in order to simplify later modification should the need arise
                match &last_arg[..] {
                    "-c" => {
                        base_cfg.config_path = path::PathBuf::from(&cur_arg[..]);
                        last_arg = String::new();
                        last_arg_multipart = false;
                        continue;
                    },
                    _ => ()
                }
            }

            // set current arg as last arg before modifying it
            last_arg = cur_arg.clone();
            // set the multipart variable flag appropriately according to the parsing strategy
            // described above this method (function? Should I be calling these functions in rust?
            // I'm honestly not sure.)
            last_arg_multipart = dash_count == 1;

            // --------------------------------------- //
            //     actually do something with args     //
            // --------------------------------------- //
            if !last_arg_multipart {
                if dash_count == 2 {
                    match &cur_arg[..] {
                        "--prefer-repo" => base_cfg.prefer_repo = true,
                        "--prefer-disk" => base_cfg.prefer_disk = true,
                        _ => (),
                    }
                } else if dash_count == 0 {
                    // if there is no primary command, the current argument has no dashes, and the last
                    // argument was not a multi-part argument, then set the primary command to the
                    // current argument
                    if base_cfg.primary_command == String::new() {
                        base_cfg.primary_command = cur_arg;
                    } else {
                        // if the previous conditions are met but there is an existing primary command,
                        // push the current argument to the primary command's argument vector
                        base_cfg.primary_command_args.push(cur_arg);
                    }
                }
            }
        }

        // ---------------------------- //
        //     sanity checks/panics     //
        // ---------------------------- //

        if base_cfg.prefer_repo && base_cfg.prefer_disk {
            panic!("cannot prefer repo and disk at the same time, exiting.")
        }

        base_cfg
    }
}
