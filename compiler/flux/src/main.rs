use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(LevelFilter::TRACE)
        .init();
    flux::run_with_args(std::env::args_os());
}
