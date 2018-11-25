use reqwest::{self, StatusCode};

pub fn healthcheck(url: &str, content: &str) {
    let mut resp = reqwest::get(url).unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = resp.text().unwrap();
    assert_eq!(text, content);
}
