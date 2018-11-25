use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use reqwest::{self, StatusCode};
pub use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;
use std::iter;
use std::time::Duration;
use std::thread;

pub fn healthcheck(url: &str, content: &str) {
    let mut resp = reqwest::get(url).unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = resp.text().unwrap();
    assert_eq!(text, content);
}

pub fn request<'a, I, J>(method: Method, path: &'a str, values: I) -> J
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
    J: for <'de> Deserialize<'de>,
{
    let params = values.into_iter().collect::<HashMap<_, _>>();
    let client = reqwest::Client::new();
    let mut resp = client.request(method, path)
        .form(&params)
        .send()
        .unwrap();

    let status = resp.status().to_owned();

    let text = resp
        .text()
        .unwrap();

    if status != StatusCode::OK {
        eprintln!("Bad Response: {}", text);
        assert_eq!(StatusCode::OK, resp.status());
    }

    serde_json::from_str(&text).unwrap()
}

pub fn rand_str() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect()
}

pub fn wait(s: u64) {
    thread::sleep(Duration::from_secs(s));
}

