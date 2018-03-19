#[derive(StructOpt, Debug)]
#[structopt(name = "dns-agent")]
pub struct Opt {
    #[structopt(short = "c", long = "config", default_value = "config.toml", help = "config file path")]
    pub config_path: String,
}
