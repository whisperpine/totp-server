//! Time-based One-time Password (TOTP) web server.
//!
//! This crate provides a web server for generating and validating TOTP tokens.

// rustc
// #![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(not(debug_assertions), deny(missing_docs))]
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), deny(warnings))]
// clippy
#![cfg_attr(not(debug_assertions), deny(clippy::todo))]
#![cfg_attr(
    not(any(test, debug_assertions)),
    deny(clippy::print_stdout, clippy::dbg_macro)
)]

#[tokio::main]
async fn main() {
    setup_panic_hook();
    init_tracing_subscriber();
    totp_server::start_server().await;
}

fn init_tracing_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", totp_server::CRATE_NAME).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Call this function in `main()` to setup panic hook.
fn setup_panic_hook() {
    use std::panic::{PanicHookInfo, set_hook};
    set_hook(Box::new(|panic_info: &PanicHookInfo| {
        // Extract the panic message.
        let message = panic_info
            .payload()
            .downcast_ref::<String>()
            .map_or("no message", |s| s);

        // Extract the location (file and line).
        let location = panic_info
            .location()
            .map_or("unknown location".to_owned(), |loc| {
                format!("{}:{}", loc.file(), loc.line())
            });

        // Log the panic with structured fields.
        tracing::error!(panic_message = message, location = location);
    }));
}
