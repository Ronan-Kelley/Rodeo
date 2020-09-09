use std::env;

#[derive(Clone, Debug)]
pub struct Options {
    command: String,
    command_args: Vec<String>,
}

impl Options {

    pub fn new(args: env::Args) -> Self {
        let mut opts = Options::default();

        for arg in args {
            if opts.command == "" {
                opts.command = arg.to_string();
            } else {
                opts.command_args.push(arg.to_string());
            }
        }

        opts
    }

    fn default() -> Self {
        Options {
            command: String::new(),
            command_args: Vec::new(),
        }
    }

}
