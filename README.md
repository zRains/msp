```rust
let server = Msp::create_with_port("grmpixelmon.com", 25565)?;

println!("{}", server.query_full()?);
```