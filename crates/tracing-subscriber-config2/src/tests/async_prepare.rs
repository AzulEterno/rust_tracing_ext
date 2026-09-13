use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::LayerConfig;
use crate::config::TracingConfig;
use crate::factory::AsyncLayerFactory;
use crate::factory::BuildFuture;
use crate::factory::BuiltLayer;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;
use crate::tests::support::layer;

struct AsyncFactory(Arc<AtomicUsize>);

impl AsyncLayerFactory for AsyncFactory {
    fn kind(&self) -> &'static str {
        "count"
    }

    fn build<'a>(&'a self, _config: &'a LayerConfig) -> BuildFuture<'a> {
        Box::pin(async move {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(BuiltLayer::new(NoopLayer))
        })
    }
}

struct NoopLayer;

impl Layer<Registry> for NoopLayer {}

#[test]
fn async_preparation_prefers_registered_async_factory() {
    let async_builds = Arc::new(AtomicUsize::new(0));
    let (sync_factory, _, sync_builds, _) = factory();
    let mut builder = TracingBuilder::new();
    builder
        .register(sync_factory)
        .expect("register sync factory");
    builder
        .register_async(AsyncFactory(Arc::clone(&async_builds)))
        .expect("register async factory");

    poll_ready(builder.prepare_async(TracingConfig {
        enabled: true,
        layers: vec![layer("count", "count", "info")],
    }))
    .expect("prepare async layers");

    assert_eq!(async_builds.load(Ordering::Relaxed), 1);
    assert_eq!(sync_builds.load(Ordering::Relaxed), 0);
}

fn poll_ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test factory unexpectedly returned a pending future"),
    }
}
