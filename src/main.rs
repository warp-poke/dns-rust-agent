extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate time;
extern crate toml;
extern crate trust_dns;
extern crate warp10;
extern crate tokio_core;
extern crate rdkafka;
extern crate futures;
extern crate futures_cpupool;

mod cli;
mod config;
mod consumer;

use std::error::Error;
use std::str::FromStr;

use structopt::StructOpt;
use tokio_core::reactor::Core;

use cli::Opt;
use config::*;
use consumer::run_core;

#[derive(Debug)]
pub struct Order {
    pub hostname: String,
    pub write_token: String,
    pub warp10_url: String,
}

pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let cfg = Config::new(&opt.config_path);
    debug!("Config read: {:#?}", cfg);

    run_core(cfg);
}
