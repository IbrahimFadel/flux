use tracing_subscriber::fmt::format::FmtSpan;

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
    flux::run_with_args(std::env::args_os());
}
