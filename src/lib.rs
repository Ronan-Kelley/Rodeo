pub mod config;

use std::fs;
use std::path;
use std::io::prelude::*;
use std::process::Command;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    #[serde(skip)]
    #[serde(default)]
    home: String,
    #[serde(skip)]
    #[serde(default)]
    config_path: String,
    pub dotfiles_directory: String,
    // difference in names here isn't huge, but naming a vector with a name that
    // implies a single value goes against my naming conventions
    #[serde(rename = "program")]
    pub programs: Vec<Program>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Program {
    pub name: String,
    pub root: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub post_deploy_cmd: String,
}

impl Settings {
    // instantiation methods //

    pub fn new_from_file(mut file: fs::File, home: String, config_path: String) -> std::io::Result<Settings> {
        // read the contents of the given file into a string
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        // using serde + toml-rs, move the config into a struct
        let mut settings: Settings = toml::from_str(&file_contents).unwrap();
        settings.home = home.clone();
        settings.config_path = config_path;
        settings.dotfiles_directory = settings.dotfiles_directory.replace("~", &home[..]);
        Ok(settings)
    }

    // methods for interacting with Program structs //
    
    pub fn deploy(self) {
        for i in self.programs.into_iter() {
            i.deploy(&self.home, &self.dotfiles_directory);
        }
    }

    pub fn collect(self) {
        for i in self.programs.into_iter() {
            i.collect(&self.home, &self.dotfiles_directory);
        }
    }

    pub fn sync_local(self) {
        for i in self.programs.into_iter() {
            i.sync_local(&self.home, &self.dotfiles_directory);
        }
    }

    pub fn sync_remote(self) {
        for i in self.clone().programs.into_iter() {
            i.sync_local(&self.home, &self.dotfiles_directory);
        }

        self.git_pull();
        self.git_push();
    }

    pub fn sync_full(self) {
        // pull before doing anything
        self.git_pull();

        for i in self.clone().programs.into_iter() {
            i.sync_local(&self.home, &self.dotfiles_directory);
        }

        self.git_push();
    }

    pub fn list_programs(&self) {
        for i in self.programs.iter() {
            println!("{}", i.name);
        }
    }

    // helper methods //
    fn git_pull(&self) {
        println!(
            "{}",
            String::from_utf8_lossy(
                Command::new("bash")
                    .arg("-c")
                    .arg("git pull")
                    .output()
                    .unwrap()
                    .stdout
                    .as_slice()
            )
        );
    }

    fn git_push(&self) {
        // pulling, committing, and pushing are all done via bash commands - while this is
        // admittedly not ideal, it has the advantage of being simple to write and simple to use,
        // automatically respecting user's git configs and, more importantly, making it very simple
        // to use features such as authentication via ssh. Additionally, it results in very
        // graceful handling of failures to push/pull/commit/etc, respecting user's git configs and
        // git's own internal logic and expectations.

        // initialize the git command outside of the command build for legibility
        let git_command = format!(
                "cd {} && \
                git pull && \
                find . -not -path \"./git\" -not -name \".\" -name \".*\" -not -name \".git*\" -not -name \"$(basename $(cat .gitmodules | grep -i \"path\" | xargs | cut -c7- | xargs))*\" -exec git add {{}} \\; && \
                git commit -m \"rodeo remote sync\" && \
                git push",
                self.dotfiles_directory
            );
       // since i can't figure out how to put a comment between the lines of a multiline string,
       // the explanation of the bash is as follows:
       //   "cd {}" (where {} is replaced by dotfiles_dir) changes the working directory to the
       //   local dotfiles repo
       //
       //   "git pull" is presumably self explanatory: performs a pull operation on the repo.
       //
       //   "find ..." is pretty chunky, but essentially it looks for every file whose name begins
       //   with a dot and doesn't match the pattern .git*, as well as trying to ignore directories
       //   that are in .gitmodules. Note that this ignores .gitignore.
       //
       //   "git commit -m \"Rodeo remote sync\" will commit all changes with the message "Rodeo
       //   remote sync"
       //
       //   finally, "git push" is probably also pretty self explanatory, as it simply pushes the
       //   changes to the remote repository.

        // build the command, simply piping git_command into the bash shell
        let command = Command::new("bash")
            .arg("-c")
            .arg(git_command)
            .output()
            .unwrap()
            .stdout;

        // print the output of git_command to terminal
        println!("{}", String::from_utf8_lossy(command.as_slice()));
    }
}

impl Program {
    
    // interprets the post-deploy command in the bash shell
    pub fn run_post_deploy_cmd(&self) -> std::io::Result<()> {
        // don't execute this method if there is no post-deploy command
        if self.post_deploy_cmd.len() < 1 {
            return Ok(())
        }

        // inform user which program's post-deploy command is being attempted
        println!("attempting to run post-deploy command for \"{}\"...", self.name);

        // run the post-deploy command, collect output into a Vec<u8>.
        // if this command fails, the error will be handled in main.
        let post_deploy_cmd = Command::new("bash")
            .arg("-c")
            .arg(&self.post_deploy_cmd[..])
            .output()?.stdout;

        // convert the post-deploy command's output from a Vec<u8> into a String
        let post_deploy_cmd_output = String::from_utf8_lossy(&post_deploy_cmd.as_slice()).to_owned();

        // give user post-deploy command's output
        println!("{}", post_deploy_cmd_output);

        Ok(())
    }
    
    // replaces all "active-duty" dotfiles from the user's system with the dotfiles in the
    // repository folder
    pub fn deploy(&self, home_dir: &String, dotfiles_dir: &String) {
        // standardize source/output dir paths
        let source_dir = Program::standardize_path(dotfiles_dir, &home_dir);
        let output_dir = Program::standardize_path(&self.root, &home_dir);

        // ensure output folder exists
        fs::create_dir_all(format!("{}/{}", output_dir, self.root.replace("~/", ""))).unwrap_or_default();

        // deploy all the files
        for i in self.paths.clone().into_iter() {
            // append the file names to the directory paths
            let in_file = format!("{}/{}/{}", source_dir, self.root.replace("~/", ""), i);
            let out_file = format!("{}/{}", output_dir, i);

            // copy the file
            Program::copy_file(in_file, out_file);
        }
    }
    
    // replaces all dotfiles in repository folder with the "active-duty" dotfiles from the user's
    // system
    pub fn collect(&self, home_dir: &String, dotfiles_dir: &String) {
        // standardize source/output dir paths
        let source_dir = Program::standardize_path(&self.root, &home_dir);
        let output_dir = Program::standardize_path(dotfiles_dir, &home_dir);

        // ensure output_dir exists
        fs::create_dir_all(format!("{}/{}", output_dir, self.root.replace("~/", ""))).unwrap_or_default();

        for i in self.paths.clone().into_iter() {
            // append the file name to the directory's path
            let in_file = format!("{}/{}", source_dir, i);
            let out_file = format!("{}/{}/{}", output_dir, self.root.replace("~/", ""), i);
            
            // copy the file
            Program::copy_file(in_file, out_file);
        }
    }

    // the in-between of copy and deploy, in which the oldest files are overwritten with the
    // newest.
    pub fn sync_local<T: Into<String>>(&self, home_dir: T, dotfiles_dir: T) {
        // convert generics to Strings
        let home_dir: String = home_dir.into();
        let dotfiles_dir: String = dotfiles_dir.into();

        // standardize paths
        let program_files_root = Program::standardize_path(&self.root, &home_dir);
        let dotfiles_dir = Program::standardize_path(dotfiles_dir, home_dir);

        // ensure directories exist
        fs::create_dir_all(&program_files_root).unwrap_or_default();
        fs::create_dir_all(format!("{}/{}", dotfiles_dir, self.root.replace("~/", ""))).unwrap_or_default();

        for i in self.paths.clone().into_iter() {
            // append the file name to the directories path. Repo file is the designation given to
            // the dotfile being pulled from the folder containing all the other dotfiles;
            // working file is the designation given to files actively in the user's filesystem in
            // their proper locations.
            let repo_file = format!("{}/{}/{}", dotfiles_dir, self.root.replace("~/", ""), i);
            let working_file = format!("{}/{}", program_files_root, i);

            // check for both files existence
            let repo_file_exists = path::Path::new(&repo_file).exists();
            let working_file_exists = path::Path::new(&working_file).exists();

            // if neither exist, don't sync
            if !repo_file_exists && !working_file_exists {
                println!("file {} does not exist in dotfiles repo or its intended place in the system, not syncing", i);
                
            // if only the repo file exists, copy the working file to repo directory
            } else if !path::Path::new(&repo_file).exists() {
                Program::copy_file(working_file, repo_file);
                continue;

            // if only the working file exists, copy the repo file to the working directory
            } else if !path::Path::new(&working_file).exists() {
                Program::copy_file(repo_file, working_file);
                continue;
            }

            // get metadata structs for both files
            let repo_file_metadata = match fs::metadata(&repo_file) {
                Ok(val) => val,
                Err(_) => {
                    println!("error syncing file \"{}\": could not access file metadata.", repo_file);
                    continue
                },
            };
            let working_file_metadata = match fs::metadata(&working_file) {
                Ok(val) => val,
                Err(_) => {
                    println!("error syncing file \"{}\": could not access file metadata.", working_file);
                    continue
                }
            };

            // get a systemtime struct for both files based on their time last modified, then
            // immediately pull the time elapsed from them
            let repo_file_modified_elapsed = match repo_file_metadata.modified() {
                Ok(val) => match val.elapsed() {
                    Ok(elapsed) => elapsed,
                    Err(_) => {
                        println!("error syncing file \"{}\": could not determine time of last modification.", i);
                        continue
                    }
                },
                Err(_) => {
                    println!("error syncing file \"{}\": could not determine time of last modification.", i);
                    continue
                }
            };

            let working_file_modified_elapsed = match working_file_metadata.modified() {
                Ok(val) => match val.elapsed() {
                    Ok(elapsed) => elapsed,
                    Err(_) => {
                        println!("error syncing file \"{}\": could not determine time of last modification.", i);
                        continue
                    }
                },
                Err(_) => {
                    println!("error syncing file \"{}\": could not determine time of last modification.", i);
                    continue
                }
            };

            // overwrite whichever file was modified a longer time ago with the more recently
            // modified file
            if repo_file_modified_elapsed < working_file_modified_elapsed {
                Program::copy_file(repo_file, working_file);
            } else if repo_file_modified_elapsed > working_file_modified_elapsed {
                Program::copy_file(working_file, repo_file);
            } else {
                println!("file \"{}\" appears to have been modified at the same time at both locations. Not syncing.", i);
            }
        }
    }

    //
    // helper functions
    //

    // copies "from" file to "to" file, outputting the given error_message string on error.
    fn copy_file<T: Into<String>>(from: T, to: T) {
        // convert all generics into Strings
        let from: String = from.into();
        let to: String = to.into();

        // ensure double slashes are ignored
        let from = from.replace("//", "/");
        let to = to.replace("//", "/");

        // copy "from" file to "to" file location
        match fs::copy(&from, &to) {
            Ok(_) => {
                println!("{} => {}", from, to);
                ()
            },
            Err(_) => println!("Error: could not perform copy operation \"{} => {}\"", from, to),
        }
    }

    // replaces ~ with the literal path of the user's home directory, and ensures that there is no
    // trailing slash.
    fn standardize_path<T: Into<String>>(path: T, home_dir: T) -> String {
        // convert both args to Strings
        let home_dir: String = home_dir.into();
        let mut path: String = path.into();

        // replace ~ with the literal home dir String, I.E. /home/<user>
        path = path.replace("~", home_dir.as_ref());
        // ensure that the final character in the String isn't a trailing slash
        match path.chars().last().unwrap_or_default() {
            '/' => path.pop().unwrap_or_default(),
            _ => '?',
        };

        // remove all instances of double slashes
        path = path.replace("//", "/");

        path
    }
}
