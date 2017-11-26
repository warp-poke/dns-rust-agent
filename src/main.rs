extern crate trust_dns;
extern crate trust_dns_resolver;
extern crate time;
extern crate warp10;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate itertools;
use std::error::Error;
use std::str::FromStr;
use trust_dns::client::*;
use trust_dns::udp::UdpClientConnection;
use trust_dns::op::Message;
use trust_dns::rr::{DNSClass, Name, RecordType};


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub hostname: String,
    pub write_token: String,
    pub warp10_url: String,
}

pub fn check(hostname: &str) -> Result<Message,Box<Error>> {
    // Construct a new Resolver with default configuration options
    //let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    // Lookup the IP addresses associated with a name.
    //let response = resolver.query(hostname,RecordType::CNAME)?;


    let address = "8.8.8.8:53".parse().unwrap();
    let conn = UdpClientConnection::new(address).unwrap();
    let client = SyncClient::new(conn);
    // Specify the name, note the final '.' which specifies it's an FQDN
    let name = Name::from_str(hostname).unwrap();

    // NOTE: see 'Setup a connection' example above
    // Send the query and get a message response, see RecordType for all supported options
    let response: Message = client.query(&name, DNSClass::ANY, RecordType::ANY).unwrap();
    
    let serialized = serde_json::to_string(&response).unwrap();
    println!("serialized = {}", serialized);
    Ok(response)
}

pub fn publish_result(order:&Order, _result:&Message
) -> Result<(),Box<Error>> {
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
        hostname: "clever-cloud.fr".to_owned(),
        write_token: "cAM7xGIjBpJcoJ5D4FDpD677YEUXiZZBQEtGt2pJl0Ewjve20HZ_Ows8Yh8Ra5KIuoo5xQZuCulOnfN0FYN6Ck98nwsDvRYBIBQkiklyfoq91KVBbJSlwVOjTv79w0srREOqhgqSSfS7kk0n21GlgeybRRJ6t342fN4f2Y89WoJ".to_owned(),
        warp10_url: "http://hv-par2-021.clvrcld.net:19001/api/v0".to_owned(),
    };
    handle_order(&order1).unwrap();
    //check(&order1.hostname);
}