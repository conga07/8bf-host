//! Replay worker requests on the main thread without unloading the filter.
use eightbf_host::{host::session::FilterHost, protocol::run_job_with_host};
use std::{io, path::Path};

fn main() {
    let requests: Vec<_> = std::env::args_os().skip(1).collect();
    if requests.is_empty() {
        eprintln!("Usage: persistent_host FIRST/request.json NEXT/request.json ...");
        std::process::exit(2);
    }
    let mut host = FilterHost::new();
    for request in requests {
        let code = run_job_with_host(Path::new(&request), Box::new(io::stdout()), &mut host);
        if code != 0 {
            std::process::exit(code);
        }
    }
}
