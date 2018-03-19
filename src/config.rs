use std::fs::File;
use std::io::*;
use std::io;
use std::env;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub broker: String,
    pub topic: String,
    pub consumer_group: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub zone: String,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let err_msg = "Missing config file are values missing";
        let cfg = load_from_path(path);

        let broker =
            env::var("BROKER").unwrap_or(cfg.as_ref().map(|c| c.broker.clone()).expect(err_msg));
        let topic =
            env::var("TOPIC").unwrap_or(cfg.as_ref().map(|c| c.topic.clone()).expect(err_msg));
        let consumer_group = env::var("CONSUMER_GROUP").unwrap_or(
            cfg.as_ref()
                .map(|c| c.consumer_group.clone())
                .expect(err_msg),
        );
        let username = env::var("USERNAME")
            .ok()
            .or(cfg.as_ref().ok().and_then(|c| c.username.clone()));
        let password: Option<String> = env::var("PASSWORD")
            .ok()
            .or(cfg.as_ref().ok().and_then(|c| c.password.clone()));
        let host = env::var("HOST").unwrap_or(cfg.as_ref().map(|c| c.host.clone()).expect(err_msg));
        let zone = env::var("ZONE").unwrap_or(cfg.as_ref().map(|c| c.zone.clone()).expect(err_msg));

        Self {
            broker,
            topic,
            consumer_group,
            username,
            password,
            host,
            zone,
        }
    }
}

pub fn load_from_path(path: &str) -> io::Result<Config> {
    let data = try!(load_file(path));

    match toml::from_str(&data) {
        Ok(config) => Ok(config),
        Err(e) => {
            error!("decoding error: {:?}", e);
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("decoding error: {:?}", e),
            ))
        }
    }
}

pub fn load_file(path: &str) -> io::Result<String> {
    let mut f = try!(File::open(path));
    let mut data = String::new();

    try!(f.read_to_string(&mut data));
    Ok(data)
}
