mod buffer;
mod logger;
pub mod response;

use futures_core::Stream;
use futures_util::stream::BoxStream;
pub use logger::*;
use servio_service::{Event, Scope, Service};
use std::collections::HashMap;
use std::io;

type BoxService<'a, ServerStream, AppStream, Error> =
    Box<dyn Service<ServerStream, AppStream = AppStream, Error = Error> + Send + 'a>;

#[derive(Default)]
pub struct ProtocolRouter<ServerStream>
where
    ServerStream: Stream<Item = Event>,
{
    services:
        HashMap<String, BoxService<'static, ServerStream, BoxStream<'static, Event>, io::Error>>,
}

impl<ServerStream> ProtocolRouter<ServerStream>
where
    ServerStream: Stream<Item = Event>,
{
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
}

impl<ServerStream> Service<ServerStream> for ProtocolRouter<ServerStream>
where
    ServerStream: Stream<Item = Event> + Send,
{
    type AppStream = BoxStream<'static, Event>;
    type Error = io::Error;

    fn call(
        &mut self,
        scope: Scope,
        server_events: ServerStream,
    ) -> Result<Self::AppStream, Self::Error> {
        if let Some(inner) = self.services.get_mut(scope.protocol()) {
            inner.call(scope, server_events)
        } else {
            Err(io::ErrorKind::Unsupported.into())
        }
    }
}
