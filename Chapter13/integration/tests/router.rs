mod utils;

use self::utils::{router as url, Method};

#[test]
fn router_healthcheck() {
    utils::healthcheck(&url("/healthcheck"), "Router Microservice");
}

#[test]
fn check_router_full() {
}
