#![allow(dead_code)]

use cookie::{Cookie, CookieJar};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
pub use reqwest::{self, Client, Method, RedirectPolicy, StatusCode};
use reqwest::header::{COOKIE, SET_COOKIE};
use serde::Deserialize;
use std::collections::HashMap;
use std::iter;
use std::time::Duration;
use std::thread;

pub struct WebApi {
    client: Client,
    url: String,
    jar: CookieJar,
}

impl WebApi {
    fn new(url: &str) -> Self {
        let client = Client::builder()
            .redirect(RedirectPolicy::none())
            .build()
            .unwrap();
        Self {
            client,
            url: url.into(),
            jar: CookieJar::new(),
        }
    }

    pub fn users() -> Self { WebApi::new(USERS) }
    pub fn mailer() -> Self { WebApi::new(MAILER) }
    pub fn content() -> Self { WebApi::new(CONTENT) }
    pub fn router() -> Self { WebApi::new(ROUTER) }

    pub fn healthcheck(&mut self, path: &str, content: &str) {
        let url = url(&self.url, path);
        let mut resp = reqwest::get(&url).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = resp.text().unwrap();
        assert_eq!(text, content);
    }

    pub fn check_status<'a, I>(&mut self, method: Method, path: &'a str, values: I, status: StatusCode)
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let url = url(&self.url, path);
        let params = values.into_iter().collect::<HashMap<_, _>>();
        let cookies = self.jar.iter()
            .map(|kv| format!("{}={}", kv.name(), kv.value()))
            .collect::<Vec<_>>()
            .join(";");
        let resp = self.client.request(method, &url)
            .header(COOKIE, cookies)
            .form(&params)
            .send()
            .unwrap();
        if let Some(value) = resp.headers().get(SET_COOKIE) {
            let raw_cookie = value.to_str().unwrap().to_owned();
            let cookie = Cookie::parse(raw_cookie).unwrap();
            self.jar.add(cookie);
        }
        assert_eq!(status, resp.status());
    }

    pub fn request<'a, I, J>(&mut self, method: Method, path: &'a str, values: I) -> J
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
        J: for <'de> Deserialize<'de>,
    {
        let url = url(&self.url, path);
        let params = values.into_iter().collect::<HashMap<_, _>>();
        let mut resp = self.client.request(method, &url)
            .form(&params)
            .send()
            .unwrap();

        let status = resp.status().to_owned();

        let text = resp
            .text()
            .unwrap();

        if status != StatusCode::OK {
            panic!("Bad response [{}] of '{}': {}", resp.status(), path, text);
        }

        let value = serde_json::from_str(&text);
        match value {
            Ok(value) => value,
            Err(err) => {
                panic!("Can't convert '{}': {}", text, err);
            },
        }
    }
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

const USERS: &str = "http://localhost:8001";

const MAILER: &str = "http://localhost:8002";

const CONTENT: &str = "http://localhost:8003";

const ROUTER: &str = "http://localhost:8000";

pub fn url(url: &str, path: &str) -> String {
    url.to_owned() + path
}
