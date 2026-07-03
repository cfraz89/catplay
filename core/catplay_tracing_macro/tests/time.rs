use catplay_tracing_macro::trace_time;

#[trace_time]
fn tracable1() {
    log::debug!("tracable1")
}

#[trace_time(warn)]
fn tracable2() {
    log::debug!("tracable2")
}

#[test]
fn test1() {
    catplay_tracing::logger::setup_test_logger(true);
    tracable1();
}

#[test]
fn test2() {
    catplay_tracing::logger::setup_test_logger(true);
    tracable2();
}
