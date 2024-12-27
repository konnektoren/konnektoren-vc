use opentelemetry::sdk::{
    propagation::TraceContextPropagator,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};

use opentelemetry::{global, KeyValue};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub async fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let enable_telemetry = env::var("ENABLE_TELEMETRY")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::default())
        .add_directive("isahc=off".parse().unwrap())
        .add_directive("tower_http=info".parse().unwrap());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_ansi(true)
        .json();

    if enable_telemetry {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let jaeger_endpoint = env::var("JAEGER_ENDPOINT")
            .unwrap_or_else(|_| "http://jaeger:14268/api/traces".to_string());
        let tracer = opentelemetry_jaeger::new_collector_pipeline()
            .with_service_name(env!("CARGO_PKG_NAME"))
            .with_endpoint(jaeger_endpoint)
            .with_reqwest()
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])),
            )
            .with_timeout(std::time::Duration::from_secs(2))
            .install_batch(opentelemetry::runtime::Tokio)?;

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .with(telemetry)
            .try_init()?;

        tracing::info!("Telemetry enabled");
    } else {
        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()?;

        tracing::info!("Telemetry disabled");
    }

    Ok(())
}
