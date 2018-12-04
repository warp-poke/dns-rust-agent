use std::str::FromStr;

use trust_dns::client::{SyncClient, Client};
use trust_dns::udp::UdpClientConnection;
use trust_dns::rr::{DNSClass, Name, RecordType, Record};
use std::result::Result;

/// From the scheduler
#[derive(Deserialize, Debug)]
pub struct DnsOrder {
    pub domain_name: String,
    pub warp10_endpoint: String,
    pub token: String,
}

#[derive(Debug)]
pub struct DnsResult {
    pub type_dns: RecordType,
    pub ttl: u32,
    pub value: String,
}

impl From<Record> for DnsResult {
    fn from(record: Record) -> Self {
        Self {
            type_dns: record.rr_type(),
            ttl: record.ttl(),
            value: record.name().to_string(),
        }
    }
}

pub fn resolve_dns(domain_name: &str, dns: &str) -> Result<Vec<Record>, String> {
    let conn = UdpClientConnection::new(dns.parse().unwrap()).unwrap();
    let client = SyncClient::new(conn);
    let name = Name::from_str(domain_name).unwrap();

    client.query(&name, DNSClass::IN, RecordType::A)
        .map_err(|e| e.to_string())
        .map(|mut res| res.take_answers())
}
