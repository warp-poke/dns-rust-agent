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
mod dns;

use structopt::StructOpt;
use tokio_core::reactor::Core;
use rdkafka::Message;
use futures::Future;
use futures::stream::Stream;
use futures_cpupool::Builder;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::stream_consumer::StreamConsumer;

use cli::Opt;
use config::*;
use dns::{resolve_dns, DnsOrder};

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

pub fn run_core(cfg: Config) {
    let mut core = Core::new().unwrap();
    let cpu_pool = Builder::new().pool_size(cfg.thread).create();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &cfg.consumer_group)
        .set("bootstrap.servers", &cfg.broker)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&[&cfg.topic]).expect("Can't subscribe to specified topic");

    let handle = core.handle();

    let processed_stream = consumer.start()
        .filter_map(|result| {
            match result {
                Ok(msg) => Some(msg),
                Err(kafka_error) => {
                    error!("Error while receiving from Kafka: {:?}", kafka_error);
                    None
                }
            }
        }).for_each(|msg| {
            let owned_message = msg.detach();
            let dns = cfg.dns.clone();
            let dns_order = serde_json::from_slice::<DnsOrder>(&owned_message.payload().unwrap()).unwrap();

            let process_message = cpu_pool.spawn_fn(move || {
                resolve_dns(&dns_order.domain_name, &dns)
            })
            .and_then(|res| {
                Ok(())
            })
            .or_else(|err| {
                error!("Error while processing message: {:?}", err);
                Ok(())
            });

            handle.spawn(process_message);
            Ok(())
        });

    info!("Thread pool running");
    core.run(processed_stream).expect("Failed to start the event loop");
}