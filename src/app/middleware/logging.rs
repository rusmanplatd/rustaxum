use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn logging_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    tower_http::trace::DefaultMakeSpan,
    tower_http::trace::DefaultOnRequest,
    tower_http::trace::DefaultOnResponse,
    tower_http::trace::DefaultOnBodyChunk,
    tower_http::trace::DefaultOnEos,
    tower_http::trace::DefaultOnFailure,
> {
    TraceLayer::new_for_http()
}