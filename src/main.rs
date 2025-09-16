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

use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    setup_panic_hook();

    if is_on_lambda() {
        init_tracing_subscriber_lambda();
        totp_server::start_server_aws_lambda().await;
    } else {
        init_tracing_subscriber();
        totp_server::start_server().await;
    }
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
        tracing::error!(location = location, "{message}");
    }));
}

fn is_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok()
}

fn init_tracing_subscriber_lambda() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(format!("{}={}", totp_server::CRATE_NAME, LevelFilter::INFO).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false) // There's decoding issue if ansi is enabled.
                .without_time(), // Time info already exists in AWS CloudWatch Logs.
        )
        .init();
}

fn init_tracing_subscriber() {
    use opentelemetry::{KeyValue, global};
    use opentelemetry_sdk::logs::SdkLoggerProvider;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_semantic_conventions as semcon;

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attribute(KeyValue::new(
            semcon::resource::SERVICE_NAME,
            totp_server::CRATE_NAME,
        ))
        .with_attribute(KeyValue::new(
            semcon::resource::SERVICE_VERSION,
            totp_server::PKG_VERSION,
        ))
        .build();

    let otel_trace_layer = {
        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .unwrap_or_else(|e| panic!("failed to build SpanExporter. error: {e}"));
        let tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(span_exporter)
            .with_resource(resource.clone())
            .build();
        global::set_tracer_provider(tracer_provider.clone());

        use opentelemetry::trace::TracerProvider;
        let tracer = tracer_provider.tracer(totp_server::CRATE_NAME);
        tracing_opentelemetry::layer().with_tracer(tracer)
    };

    let otel_metrics_layer = {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .build()
            .unwrap_or_else(|e| panic!("failed to build MetricExporter. error: {e}"));
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter)
            .with_resource(resource.clone())
            .build();
        global::set_meter_provider(meter_provider.clone());
        tracing_opentelemetry::MetricsLayer::new(meter_provider)
    };

    let otel_log_layer = {
        let log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .build()
            .unwrap_or_else(|e| panic!("failed to build LogExporter. error: {e}"));
        let provider = SdkLoggerProvider::builder()
            .with_simple_exporter(log_exporter)
            .with_resource(resource.clone())
            .build();
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&provider)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(format!("{}={}", totp_server::CRATE_NAME, LevelFilter::INFO).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(otel_trace_layer)
        .with(otel_metrics_layer)
        .with(otel_log_layer)
        .init();
}
