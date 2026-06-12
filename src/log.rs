use tracing_subscriber::{
    EnvFilter, Layer, fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

const DEBUG_LOG_FILTER: &str = "dreamstack=trace,eframe=warn,egui=warn,wgpu=warn,naga=warn";
const RELEASE_LOG_FILTER: &str = "dreamstack=info,eframe=warn,egui=warn,wgpu=warn,naga=warn";

pub(crate) fn init_tracing() {
    let filter = if cfg!(debug_assertions) {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEBUG_LOG_FILTER))
    } else {
        EnvFilter::new(RELEASE_LOG_FILTER)
    };

    let terminal = layer().with_ansi(true).with_filter(filter);

    registry().with(terminal).init();
}
