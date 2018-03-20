use std::str::FromStr;

use trust_dns::client::*;
use trust_dns::udp::UdpClientConnection;
use trust_dns::rr::{DNSClass, Name, RecordType};
use trust_dns::op::Message as OpMessage;
use time;
use warp10;

/// From the scheduler
#[derive(Deserialize, Debug)]
pub struct DnsOrder {
    pub domain_name: String,
    warp10_endpoint: String,
    token: String,
}

impl From<DnsOrder> for Vec<warp10::Data> {
    fn from(item: DnsOrder) -> Self {
        vec![
            warp10::Data::new(
                time::now_utc().to_timespec(),
                None,
                "dns.agent".to_string(),
                Vec::new(),
                warp10::Value::Int(0),
            )
        ]
    }
}

pub fn resolve_dns(domain_name: &str, dns: &str) -> Result<OpMessage, String> {
    let conn = UdpClientConnection::new(dns.parse().unwrap()).unwrap();
    let client = SyncClient::new(conn);
    let name = Name::from_str(domain_name).unwrap();
    let response = client.query(&name, DNSClass::IN, RecordType::A).unwrap();
    debug!("DNS client response: {:#?}", response);

    Ok(response)
}