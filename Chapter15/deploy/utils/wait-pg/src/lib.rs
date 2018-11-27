extern crate failure;
extern crate postgres;


pub fn wait_pg(url: &str) {
    use postgres::{Connection, TlsMode};
    while Connection::connect("postgres://postgres@localhost:5433", TlsMode::None).is_err() { }
}
