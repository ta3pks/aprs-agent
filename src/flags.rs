use clap::Parser;

#[derive(Debug, Parser)]
pub struct Flags {
    /// The path to the config file
    #[arg(short, long, default_value = "./aprsconfig.ini")]
    pub config: String,
    #[arg(short, long)]
    pub write_default_config: bool,
}

static mut FLAGS: Option<Flags> = None;
pub fn parse() {
    unsafe {
        FLAGS = Some(Flags::parse());
    }
}
pub fn flags() -> &'static Flags {
    unsafe { FLAGS.as_ref().expect("flags are not initially parsed") }
}
