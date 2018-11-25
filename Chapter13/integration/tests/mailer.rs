mod utils;

use self::utils::{mailer as url, Method};

#[test]
fn mails_healthcheck() {
    utils::healthcheck(&url("/"), "Mailer Microservice");
}

#[test]
fn send_mail() {
    let email = utils::rand_str() + "@example.com";
    let code = utils::rand_str();
    let params = vec![
        ("to", email.as_ref()),
        ("code", code.as_ref()),
    ];
    let sent: bool = utils::request(Method::POST, &url("/send"), params);
    assert!(sent);
}
