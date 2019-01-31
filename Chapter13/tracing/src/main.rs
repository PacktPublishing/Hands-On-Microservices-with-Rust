use rustracing::sampler::AllSampler;
use rustracing::tag::Tag;
use rustracing_jaeger::Tracer;
use rustracing_jaeger::reporter::JaegerCompactReporter;
use std::time::Duration;
use std::thread;

fn wait(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn main() {
    let (tracer1, span_rx1) = Tracer::new(AllSampler);
    let (tracer2, span_rx2) = Tracer::new(AllSampler);
    thread::spawn(move || {
        loop {
            {
                let req_span = tracer1
                    .span("incoming request")
                    .start();
                wait(50);
                {
                    let db_span = tracer2
                        .span("database query")
                        .child_of(&req_span)
                        .tag(Tag::new("query", "SELECT column FROM table;"))
                        .start();
                    wait(100);
                    let _resp_span = tracer2
                        .span("generating response")
                        .follows_from(&db_span)
                        .tag(Tag::new("user_id", "1234"))
                        .start();
                    wait(10);
                }
            }
            wait(150);
        }
    });

    let reporter1 = JaegerCompactReporter::new("router").unwrap();
    let reporter2 = JaegerCompactReporter::new("dbaccess").unwrap();
    loop {
        if let Ok(span) = span_rx1.try_recv() {
            reporter1.report(&[span]).unwrap();
        }
        if let Ok(span) = span_rx2.try_recv() {
            reporter2.report(&[span]).unwrap();
        }
        thread::yield_now();
    }
}
