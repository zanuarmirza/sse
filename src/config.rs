struct Broker {
    host: String,
    port: usize,
}

pub struct Config {
    broker: Broker,
}
