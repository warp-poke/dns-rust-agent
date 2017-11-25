extern crate trust_dns;
extern crate trust_dns_resolver;
extern crate time;
extern crate warp10;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate itertools;
use std::error::Error;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::lookup::Lookup;
use trust_dns::rr::record_type::RecordType;
use trust_dns_resolver::config::*;

pub struct Order {
    pub hostname: String,
    pub write_token: String,
    pub warp10_url: String,
}

pub fn check(hostname: &str) -> Result<Lookup,Box<Error>> {
    // Construct a new Resolver with default configuration options
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    // Lookup the IP addresses associated with a name.
    let response = resolver.lookup(hostname,RecordType::ANY)?;
    Ok(response)
}

pub fn publish_result(order:&Order, result:&Lookup) -> Result<(),Box<Error>> {
    let client = warp10::Client::new(&order.warp10_url)?;
    let writer = client.get_writer(order.write_token.to_owned());
    writer.post(vec![
        warp10::Data::new(
            time::now_utc().to_timespec(),
            None,
            "poke.dns.lookup".to_string(),
            vec![
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
    let order1 = Order{
        hostname: "www.clever-cloud.com".to_owned(),
        write_token: "cAM7xGIjBpJcoJ5D4FDpD677YEUXiZZBQEtGt2pJl0Ewjve20HZ_Ows8Yh8Ra5KIuoo5xQZuCulOnfN0FYN6Ck98nwsDvRYBIBQkiklyfoq91KVBbJSlwVOjTv79w0srREOqhgqSSfS7kk0n21GlgeybRRJ6t342fN4f2Y89WoJ".to_owned(),
        warp10_url: "http://hv-par2-021.clvrcld.net:19001/api/v0".to_owned(),
    };
    handle_order(&order1).unwrap();
}