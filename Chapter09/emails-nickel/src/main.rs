extern crate failure;
#[macro_use]
extern crate nickel;
extern crate lettre;

use failure::{format_err, Error};
use lettre::{SendableEmail, EmailAddress, Envelope, SmtpClient, SmtpTransport, Transport};
use nickel::{Action, Nickel, HttpRouter, FormBody, Request, Response, MiddlewareResult};
use nickel::status::StatusCode;
use nickel::template_cache::{ReloadPolicy, TemplateCache};
use std::collections::HashMap;
use std::thread;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender};

fn spawn_sender() -> Sender<SendableEmail> {
    let (tx, rx) = channel();
    let client = SmtpClient::new_unencrypted_localhost()
        .expect("can't start smtp client");
    thread::spawn(move || {
        let mut mailer = SmtpTransport::new(client);
        for email in rx.iter() {
            let result = mailer.send(email);
            if let Err(err) = result {
                println!("Can't send mail: {}", err);
            }
        }
        mailer.close();
    });
    tx
}

fn send_impl(req: &mut Request<Data>) -> Result<(), Error> {
    let (to, code) = {
        let params = req.form_body().map_err(|_| format_err!(""))?;
        let to = params.get("to").ok_or(format_err!("to field not set"))?.to_owned();
        let code = params.get("code").ok_or(format_err!("code field not set"))?.to_owned();
        (to, code)
    };
    let data = req.server_data();
    let to = EmailAddress::new(to.to_owned())?;
    let envelope = Envelope::new(None, vec![to])?;
    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("code", &code);
    let mut body: Vec<u8> = Vec::new();
    data.cache.render("templates/confirm.tpl", &mut body, &params)?;
    let email = SendableEmail::new(envelope, "Confirm email".to_string(), Vec::new());
    let sender = data.sender.lock().unwrap().clone();
    sender.send(email).map_err(|_| format_err!("can't send email"))?;
    Ok(())
}

fn send<'mw>(req: &mut Request<Data>, mut res: Response<'mw, Data>) -> MiddlewareResult<'mw, Data> {
    try_with!(res, send_impl(req).map_err(|_| StatusCode::BadRequest));
    res.set(StatusCode::Ok);
    Ok(Action::Continue(res))
}

struct Data {
    sender: Mutex<Sender<SendableEmail>>,
    cache: TemplateCache,
}

fn main() {
    let tx = spawn_sender();

    let data = Data {
        sender: Mutex::new(tx),
        cache: TemplateCache::with_policy(ReloadPolicy::Always),
    };

    let mut server = Nickel::with_data(data);
    server.get("/", middleware!("Mailer Microservice"));
    server.post("/send", send);
    server.listen("0.0.0.0:7000").unwrap();
}
