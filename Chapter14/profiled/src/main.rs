use std::fs::File;

pub fn main() {
    {
        let _req_span = flame::start_guard("incoming request");
        {
            let _db_span = flame::start_guard("database query");
            let _resp_span = flame::start_guard("generating response");
        }
    }

    flame::dump_html(&mut File::create("out.html").unwrap()).unwrap();
    flame::dump_json(&mut File::create("out.json").unwrap()).unwrap();
    flame::dump_stdout();
}
