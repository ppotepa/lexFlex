//! Trace digest emission on translation pipeline.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_trace_digest_stages_on_translate_in_process() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let capture = buf.clone();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("lexflex=trace"))
        .with_writer(move || CaptureWriter(capture.clone()))
        .with_ansi(false)
        .try_init();

    let api = lexflex::api::LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("api");
    let out = api
        .translate("Tomek poszedł.", "pl", "en")
        .expect("translate");
    assert_eq!(out, "Tomek went.");

    let bytes = buf.lock().unwrap().clone();
    let log = String::from_utf8_lossy(&bytes);
    assert!(
        log.contains("lexflex_semantic_digest") && log.contains("stage="),
        "expected digest events in captured log: {log}"
    );
    for stage in ["deduction", "discourse", "parse_complete", "pre_generation"] {
        assert!(log.contains(stage), "missing stage={stage}");
    }
}

