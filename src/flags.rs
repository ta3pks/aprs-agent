use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Flags {
    /// The path to the config file
    #[arg(short, long, default_value = "./aprsconfig.toml")]
    pub config: String,
    /// Write the default config to the config file and exit
    #[arg(short, long)]
    pub write_default_config: bool,
    /// Print the config and exit
    #[arg(short, long)]
    pub print_config: bool,
}

static mut FLAGS: Option<Flags> = None;
pub fn parse() -> Flags {
    let flags = Flags::parse();
    unsafe {
        FLAGS = Some(flags.clone());
    }
    flags
}
pub fn flags() -> &'static Flags {
    unsafe { FLAGS.as_ref().expect("flags are not initially parsed") }
}
