use rodeo::*;
use std::fs::File;
use std::fs;
use std::env;

fn main() -> std::io::Result<()> {

    // |--------------------------------------------|
    // | config file creation/reading/deserializing |
    // |--------------------------------------------|

    // reads $HOME variable, returns home directory's location without a trailing slash
    let user_home = env::var("HOME").expect("Could not get path of user's home directory!");
    // get config file path, by default ~/.config/rodeo/rodeo.toml
    let mut config_file_path = match env::var("RODEO_PATH") {
        Ok(val) => val,
        Err(_) => format!("{}/.config/rodeo/rodeo.toml", user_home)
    };

    // check for existance of config file. If it exists, open it, if not, fall back on default
    // location.
    let config_file = match File::open(&config_file_path) {
        // if the file at the given path is invalid or does not exist, attempt to use the default
        // path
        Ok(val) => val,
        Err(_) => {
            // inform user of the error, attempt to use the default path. This will ensure the
            // existance of a folder at ~/.config/rodeo if ~/.config exists and user has
            // permissions. If ~/.config does not exist or the user does not have permissions, the
            // program will panic. Finally, if rodeo.toml exists it will be opened and loaded.
            // If no rodeo.toml file exists at that location, it will be created.
            println!("no path \"{}\", attempting to use default config path...", &config_file_path);
            config_file_path = format!("{}/.config/rodeo", user_home);
            match fs::create_dir(&config_file_path) {
                Ok(_) => File::create(&config_file_path)?,
                Err(_) => match File::create(&config_file_path) {
                    Ok(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        panic!("created a new config file at ~/.config/rodeo/rodeo.toml, you must populate it for rodeo to function.")
                    }
                    Err(_) => panic!("could not read config file at $RODEO_PATH, ~/.config/rodeo/rodeo.toml, or create folder ~/.config/rodeo, exiting!")
                }
            }
        }
    };

    // read the user's config file
    let settings = Settings::new_from_file(config_file, user_home, config_file_path)?;

    // |----------------------------------|
    // | command interpretation/execution |
    // |----------------------------------|

    let mut args = env::args();

    let command = if args.len() > 1 {
        args.nth(1).unwrap_or("none".to_owned())
    } else {
        println!("no command provided. Stop.");
        "none".to_owned()
    };

    match &command[..] {
        "deploy" | "d" => settings.deploy(),
        "collect" | "c" => settings.collect(),
        "sync-local" | "sync_local" | "local_sync" | "local-sync" | "lsync" => settings.sync_local(),
        "sync-remote" | "sync_remote" | "remote_sync" | "remote-sync" | "rsync" => settings.sync_remote(),
        "sync-full" | "sync_full" | "full_sync" | "full-sync" | "fsync" => settings.sync_full(),
        "help" | "h" => print_help(),
        _ => println!("invalid command \"{}\". Stop.", command),
    }

    Ok(())
}

pub fn print_help() {
    println!("\
        deploy | d:\n\tdeploys all dotfiles referenced in your config in your local dotfiles repo to your system, overwriting existing files\n\n\
        collect | c:\n\tcollects all dotfiles referenced in your config that are active in your system to your local dotfiles repo, overwriting existing files\n\n\
        sync[-/_]local | local[-/_]sync | lsync:\n\treplaces all dotfiles referenced in your config, both deployed and collected, with whichever respective file was last modified\n\n\
        sync[-/_]remote | remote[-/_]sync | rsync:\n\tpulls, commits changes, and pushes in the context of your local repo's directory. Works with any git repository.\n\n\
        sync[-/_]full | full[-/_]sync | fsync:\n\tpulls from remote git repo, runs a local sync, then commits changes and pushes back up to remote. Works with any git repository.\n\n\
        help | h:\n\tprint this text\n\
        ");
}
