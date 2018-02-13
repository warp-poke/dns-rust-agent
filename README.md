# DNS Agent

Make use of [trust_dns](https://docs.rs/trust-dns/0.13.0/trust_dns/) to test DNS of domain, and push results in warp10.

### Run

	cargo run

### Needed env variables

For now, everything is hardcoded (`hostname`,`write_token`,`warp10_url` and `address`).
