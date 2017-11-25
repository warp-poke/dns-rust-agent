extern crate trust_dns_resolver;
extern crate time;
extern crate warp10;
#[macro_use]
extern crate log;
extern crate env_logger;
use std::error::Error;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::lookup_ip::LookupIp;
use trust_dns_resolver::config::*;

pub struct Order {
    pub hostname: String,
    pub write_token: String,
    pub service_id: String,
    pub warp10_url: String,
}

pub fn check(hostname: &str) -> Result<LookupIp,Box<Error>> {
    // Construct a new Resolver with default configuration options
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    // Lookup the IP addresses associated with a name.
    let response = resolver.lookup_ip(hostname)?;
    Ok(response)
}

pub fn publish_result(order:&Order, _result:&LookupIp) -> Result<(),Box<Error>> {
    let client = warp10::Client::new(&order.warp10_url)?;
    let writer = client.get_writer(order.write_token.to_owned());
    writer.post(vec![
        warp10::Data::new(
            time::now_utc().to_timespec(),
            None,
            "poke.dns.lookup".to_string(),
            vec![
                warp10::Label::new("service_id", &order.service_id)
            ],
            warp10::Value::String(unimplemented!())
        )
    ])?;
    Ok(())
}

pub fn handle_order(order: &Order) -> Result <(),Box<Error>> {
    let result = check(&order.hostname)?;
    publish_result(order,&result)?;
    info!("published result of {:?} in warp10", result);
    Ok(())
}

pub fn main(){
    let response = check("domain.par.clever-cloud.com");
    println!("{:?}",response);
}