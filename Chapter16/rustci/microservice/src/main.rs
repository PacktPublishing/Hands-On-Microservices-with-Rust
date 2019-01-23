use actix_web::{middleware, server, App, HttpRequest};

fn index(_req: &HttpRequest) -> &'static str {
    "Microservice Updated"
}

fn main() {
    env_logger::init();
    let sys = actix::System::new("microservice");
    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.f(index))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();
    let _ = sys.run();
}
