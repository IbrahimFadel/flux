use tracing_subscriber::{prelude::*, Registry};
use tracing_tree::HierarchicalLayer;

fn main() {
    if cfg!(debug_assertions) {
        let subscriber = Registry::default().with(HierarchicalLayer::new(2));
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
    flux::build();
}
