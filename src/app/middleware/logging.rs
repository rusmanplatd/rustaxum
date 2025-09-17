use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn logging_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &axum::extract::Request| {
            tracing::info_span!(
                "http_request",
                method = ?request.method(),
                uri = ?request.uri(),
            )
        })
        .on_response(|response: &axum::response::Response, latency: std::time::Duration, _span: &tracing::Span| {
            tracing::info!(
                status = response.status().as_u16(),
                latency = ?latency,
                "response"
            );
        })
}