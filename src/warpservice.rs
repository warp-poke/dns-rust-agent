use warp10::{Data, Client, Error as WarpErr, Label, Value};
use time::{Timespec, now_utc};

use dns::DnsResult;

#[derive(Debug)]
pub struct DnsQueryResults {
    pub dns_results: Vec<DnsResult>,
    pub timestamp: Timespec,
    pub labels: Vec<Label>,
}

impl DnsQueryResults {
    pub fn new(dns_results: Vec<DnsResult>) -> Self {
        Self {
            dns_results,
            timestamp: now_utc().to_timespec(),
            labels: Vec::with_capacity(3),
        }
    }
}

impl Into<Vec<Data>> for DnsQueryResults {
    fn into(self) -> Vec<Data> {
        let mut res = Vec::with_capacity(self.dns_results.len() * 3);

        for dns_res in self.dns_results.into_iter() {
            res.push(Data::new(
                self.timestamp,
                None,
                "dns.type".to_string(),
                self.labels.clone(),
                Value::String(dns_res.type_dns.to_string()),
            ));

            res.push(Data::new(
                self.timestamp,
                None,
                "dns.ttl".to_string(),
                self.labels.clone(),
                Value::Int(dns_res.ttl as i32),
            ));

            res.push(Data::new(
                self.timestamp,
                None,
                "dns.value".to_string(),
                self.labels.clone(),
                Value::String(dns_res.value),
            ));
        }
        res
    }
}

pub fn warp10_post(dns_query_results: Vec<Data>, url: String, write_token: String) -> Result<(), WarpErr> {
    trace!("Writing to warp10 {}", &url);

    let client = Client::new(&url)?;
    let writer = client.get_writer(write_token);

    writer.post(dns_query_results)
        .map(|_| ())
}