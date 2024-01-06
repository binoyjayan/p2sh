use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Enables filter mode
    #[arg(default_value_t = false, short, long)]
    filter: bool,
    /// Script to run
    script: Option<String>,
    /// Script arguments
    args: Vec<String>,
}

pub struct CliArgs {
    args: Vec<String>,
    filter: bool,
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
            filter: cliargs.filter,
        }
    }
    pub fn is_filter(&self) -> bool {
        self.filter
    }
    pub fn get_args(&self) -> &[String] {
        self.args.as_slice()
    }
}
