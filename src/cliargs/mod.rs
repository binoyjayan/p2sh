use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Program via command line argument
    #[arg(short, long)]
    command: Option<String>,
    /// Script to run
    script: Option<String>,
    /// Script arguments
    args: Vec<String>,
}

pub struct CliArgs {
    args: Vec<String>,
    cmd: Option<String>,
}

impl CliArgs {
    pub fn new() -> Self {
        let cliargs = Args::parse();
        // Create args with script as first element
        let mut args = Vec::new();
        if let Some(script) = cliargs.script.clone() {
            args.push(script);
        }
        // Extend args with script arguments
        args.extend_from_slice(cliargs.args.as_slice());

        Self {
            args,
            cmd: cliargs.command,
        }
    }
    pub fn get_args(&self) -> &[String] {
        self.args.as_slice()
    }
    pub fn get_cmd(&self) -> Option<String> {
        self.cmd.clone()
    }
}
