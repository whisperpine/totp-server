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

use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::LazyLock;
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
        OTEL_SDK_PROVIDER.force_flush().unwrap_or_else(|e| {
            panic!("failed to force_flush opentelemetry sdk providers. error: {e}")
        });
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

static OTEL_SDK_PROVIDER: LazyLock<OtelSdkProviders> = LazyLock::new(init_opentelemetry);

fn init_opentelemetry() -> OtelSdkProviders {
    use opentelemetry::KeyValue;
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
    init_propagator();
    OtelSdkProviders {
        logger: init_logger(&resource),
        meter: init_meter(&resource),
        tracer: init_tracer(&resource),
    }
}

struct OtelSdkProviders {
    logger: SdkLoggerProvider,
    meter: SdkMeterProvider,
    tracer: SdkTracerProvider,
}

impl OtelSdkProviders {
    fn force_flush(&self) -> Result<(), OTelSdkError> {
        self.logger.force_flush()?;
        self.meter.force_flush()?;
        self.tracer.force_flush()?;
        Ok(())
    }
}

fn init_logger(resource: &opentelemetry_sdk::Resource) -> SdkLoggerProvider {
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .unwrap_or_else(|e| panic!("failed to build LogExporter. error: {e}"));
    SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource.clone())
        .build()
}

fn init_meter(resource: &opentelemetry_sdk::Resource) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .build()
        .unwrap_or_else(|e| panic!("failed to build MetricExporter. error: {e}"));
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource.clone())
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    meter_provider
}

fn init_tracer(resource: &opentelemetry_sdk::Resource) -> SdkTracerProvider {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap_or_else(|e| panic!("failed to build SpanExporter. error: {e}"));
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    tracer_provider
}

fn init_propagator() {
    use opentelemetry::propagation::{TextMapCompositePropagator, TextMapPropagator};
    let propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>> = vec![
        Box::new(opentelemetry_sdk::propagation::TraceContextPropagator::new()),
        Box::new(opentelemetry_sdk::propagation::BaggagePropagator::new()),
    ];
    let composite_propagator = TextMapCompositePropagator::new(propagators);
    opentelemetry::global::set_text_map_propagator(composite_propagator);
}

fn init_tracing_subscriber() {
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use tracing_opentelemetry::MetricsLayer;

    let otel_log_layer = OpenTelemetryTracingBridge::new(&OTEL_SDK_PROVIDER.logger);
    let otel_metrics_layer = MetricsLayer::new(OTEL_SDK_PROVIDER.meter.clone());
    let otel_trace_layer = {
        use opentelemetry::trace::TracerProvider;
        let tracer = OTEL_SDK_PROVIDER.tracer.tracer(totp_server::CRATE_NAME);
        tracing_opentelemetry::layer().with_tracer(tracer)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(format!("{}={}", totp_server::CRATE_NAME, LevelFilter::INFO).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(otel_log_layer)
        .with(otel_metrics_layer)
        .with(otel_trace_layer)
        .init();
}
