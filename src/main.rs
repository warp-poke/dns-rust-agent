#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;
extern crate warp10;
extern crate trust_dns;
extern crate trust_dns_resolver;

use std::error::Error;
use std::str::FromStr;

use trust_dns::client::*;
use trust_dns::udp::UdpClientConnection;
use trust_dns::op::Message;
use trust_dns::rr::{DNSClass, Name, RecordType};

#[derive(Debug)]
pub struct Order {
    pub hostname: String,
    pub write_token: String,
    pub warp10_url: String,
}

pub fn check(hostname: &str) -> Result<Message, Box<Error>> {
    let address = "8.8.8.8:53".parse().unwrap();
    let conn = UdpClientConnection::new(address).unwrap();
    let client = SyncClient::new(conn);
    // Specify the name, note the final '.' which specifies it's an FQDN
    let name = Name::from_str(hostname).unwrap();

    let response: Message = client.query(&name, DNSClass::ANY, RecordType::ANY).unwrap();
    trace!("{} response: {:#?}", hostname, response);

    Ok(response)
}

pub fn publish_result(order: &Order, _result: &Message) -> Result<(), Box<Error>> {
    let client = warp10::Client::new(&order.warp10_url)?;
    let writer = client.get_writer(order.write_token.to_owned());
    writer.post(vec![
        warp10::Data::new(
            time::now_utc().to_timespec(),
            None,
            "poke.dns.lookup".to_string(),
            vec![],
            warp10::Value::String(unimplemented!()),
        ),
    ])?;
    Ok(())
}

pub fn handle_order(order: &Order) -> Result<(), Box<Error>> {
    let result = check(&order.hostname)?;
    publish_result(order, &result)?;
    info!("published result of {:?} in warp10", result);
    Ok(())
}

pub fn main() {
    env_logger::init();

    let order1 = Order{
        hostname: "clever-cloud.fr".to_owned(),
        write_token: "cAM7xGIjBpJcoJ5D4FDpD677YEUXiZZBQEtGt2pJl0Ewjve20HZ_Ows8Yh8Ra5KIuoo5xQZuCulOnfN0FYN6Ck98nwsDvRYBIBQkiklyfoq91KVBbJSlwVOjTv79w0srREOqhgqSSfS7kk0n21GlgeybRRJ6t342fN4f2Y89WoJ".to_owned(),
        warp10_url: "http://hv-par2-021.clvrcld.net:19001/api/v0".to_owned(),
    };
    handle_order(&order1).unwrap();
}
