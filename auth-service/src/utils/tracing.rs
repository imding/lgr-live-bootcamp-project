use {
    axum::{body::Body, extract::Request, response::Response},
    color_eyre::eyre::Result,
    std::time::Duration,
    tracing::{Level, Span, event, field, span},
    tracing_error::ErrorLayer,
    tracing_subscriber::{EnvFilter, fmt, prelude::*, registry},
    uuid::Uuid,
};

pub fn init_tracing() -> Result<()> {
    let format_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    registry().with(filter_layer).with(format_layer).with(ErrorLayer::default()).init();

    Ok(())
}

pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = Uuid::new_v4();

    span!(
        Level::INFO,
        "[REQUEST]",
        method = field::display(request.method()),
        uri = field::display(request.uri()),
        version = field::debug(request.version()),
        request_id = field::display(request_id)
    )
}

pub fn on_request(_request: &Request<Body>, _span: &Span) {
    event!(Level::INFO, "[REQUEST START]");
}

pub fn on_response(response: &Response, latency: Duration, _span: &Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let status_code_class = status_code / 100;

    match status_code_class {
        4..=5 => event!(
            Level::ERROR,
            latency = ?latency,
            status = status_code,
            "[REQUEST END]"
        ),
        _ => event!(
            Level::INFO,
            latency = ?latency,
            status = status_code,
            "[REQUEST END]"
        ),
    };
}
