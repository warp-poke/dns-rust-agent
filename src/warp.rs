use warp10;
use std;

pub fn warp10_post(data: Vec<warp10::Data>, url: String, write_token: String)
 -> std::result::Result<warp10::Response, warp10::Error> {
    trace!("Writing to warp10 {}", &url);
    let client = warp10::Client::new(&url)?;
    let writer = client.get_writer(write_token);
    let res = writer.post(data)?;
    Ok(res)
}