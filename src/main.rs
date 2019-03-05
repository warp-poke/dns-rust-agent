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
extern crate warp;
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
extern crate warp10;
extern crate tokio;
extern crate rdkafka;
extern crate futures;
extern crate futures_cpupool;

mod cli;
mod config;
mod dns;
mod warpservice;

use std::error::Error;
use std::thread;

use structopt::StructOpt;
use tokio::executor::current_thread::CurrentThread;
use rdkafka::Message;
use futures::Future;
use futures::stream::Stream;
use futures_cpupool::Builder;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use lazy_static::lazy_static;
use prometheus::{Encoder, IntGauge, opts, TextEncoder, register_int_gauge};
use warp::{Filter, path, serve};

use cli::Opt;
use config::*;
use dns::*;
use warpservice::{warp10_post, DnsQueryResults};
use warp10::Label;

lazy_static! {
    static ref KAFKA_ERROR_GAUGE: IntGauge = register_int_gauge!(opts!(
        "dns_agent_kafka_error_events",
        "Number of events causing kafka consumer error"
    ))
    .expect("create dns_agent_kafka_error_events metrics");

    static ref PROCESS_GAUGE: IntGauge = register_int_gauge!(opts!(
        "dns_agent_processed_events",
        "Number of events processed"
    ))
    .expect("create dns_agent_processed_events metrics");
    
    static ref ERROR_GAUGE: IntGauge = register_int_gauge!(opts!(
        "dns_agent_error_events",
        "Number of events causing error when processed"
    ))
    .expect("create dns_agent_error_events metrics");

     static ref SUCCESS_GAUGE: IntGauge = register_int_gauge!(opts!(
        "dns_agent_success_events",
        "Number of events correctly handled"
    ))
    .expect("create dns_agent_success_events metrics");
}

pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let cfg = Config::new(&opt.config_path);
    debug!("Config read: {:#?}", cfg);

    KAFKA_ERROR_GAUGE.set(0);
    PROCESS_GAUGE.set(0);
    ERROR_GAUGE.set(0);
    SUCCESS_GAUGE.set(0);

    let metrics_server = path!("metrics")
        .map(|| {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            buffer
        });

    let metrics_thread = thread::spawn(move || { serve(metrics_server).run(([127, 0, 0, 1], 3030)); });
    run_core(cfg);
    metrics_thread.join().expect("stop metrics server thread without troubles");
}

pub fn run_core(cfg: Config) {
    let mut io_loop = CurrentThread::new();
    let cpu_pool = Builder::new().pool_size(cfg.thread).create();

    let mut consumer_builder = ClientConfig::new();

    if let (Some(ref user), Some(ref pass)) = (cfg.username.as_ref(), cfg.password.as_ref()) {
        consumer_builder
          .set("security.protocol", "SASL_SSL")
          .set("sasl.mechanisms", "PLAIN")
          .set("sasl.username", &user)
          .set("sasl.password", &pass);
    }

    let consumer = consumer_builder
        .set("group.id", &cfg.consumer_group)
        .set("bootstrap.servers", &cfg.broker)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create::<StreamConsumer<_>>()
        .expect("Consumer creation failed");

    consumer.subscribe(&[&cfg.topic]).expect("Can't subscribe to specified topic");

    let handle = io_loop.handle();

    let processed_stream = consumer.start()
        .filter_map(|result| {
            match result {
                Ok(msg) => Some(msg),
                Err(kafka_error) => {
                    KAFKA_ERROR_GAUGE.add(1);
                    error!("Error while receiving from Kafka: {:?}", kafka_error);
                    None
                }
            }
        }).for_each(|msg| {

            PROCESS_GAUGE.add(1);
            let owned_message = msg.detach();
            let dns = cfg.dns.clone();
            let zone = cfg.host.clone();
            let host = cfg.host.clone();

            let dns_order = serde_json::from_slice::<DnsOrder>(&owned_message.payload().unwrap()).unwrap();
            let domain_name = dns_order.domain_name.clone();
            let warp10_endpoint =  dns_order.warp10_endpoint.clone();
            let write_token = dns_order.token.clone();

            let process_message = cpu_pool.spawn_fn(move || {
                resolve_dns(&domain_name, &dns)
            })
            .and_then(move |records| {
                let dns_results = records.into_iter()
                                        .map(|r| DnsResult::from(r))
                                        .collect();

                let mut query_result = DnsQueryResults::new(dns_results);
                query_result.labels.push(Label::new("zone", &zone));
                query_result.labels.push(Label::new("host", &host));

                warp10_post(query_result.into(), warp10_endpoint, write_token)
                    .map_err(|e| e.description().to_string())
            })
            .or_else(|err| {
                ERROR_GAUGE.add(1);
                error!("Error while processing message: {:?}", err);
                Ok(())
            });

            handle.spawn(process_message).unwrap();
            SUCCESS_GAUGE.add(1);
            Ok(())
        });

    info!("Thread pool running");
    io_loop.block_on(processed_stream).expect("Failed to start the event loop");
}
