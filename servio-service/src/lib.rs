use futures_core::Stream;
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

/// Request metadata, containing protocol identifier and a set of scopes, identified by type.
///
/// It works like a TypeMap with a tag. A typical way to use the scope is to match protocol
/// identifier by value and get one or multiple scopes.
/// `Scope` is itself protocol-independent with protocol identifiers and scope types provided in
/// other crates. For example, HTTP and WebSocket are defined in servio-http crate.
#[derive(Clone, Debug)]
pub struct Scope {
    protocol: Cow<'static, str>,
    scopes: HashMap<TypeId, Arc<dyn Any + Sync + Send>>,
}

impl Scope {
    /// Creates a new `Scope` with a specified protocol identifier.
    pub fn new(protocol: Cow<'static, str>) -> Self {
        Self {
            protocol,
            scopes: Default::default(),
        }
    }

    /// Returns a protocol identifier of `Scope`.
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Returns new `Scope` with specified prococol identifier, consuming surrent `Scope`.
    pub fn with_protocol(self, protocol: Cow<'static, str>) -> Self {
        Self {
            protocol,
            scopes: self.scopes,
        }
    }

    /// Returns reference-counted scope of provided type.
    /// This scope can be saved by middleware for internal use.
    pub fn get<T: Any + Sync + Send>(&self) -> Option<Arc<T>> {
        self.scopes
            .get(&TypeId::of::<T>())?
            .clone()
            .downcast::<T>()
            .ok()
    }

    /// Inserts a scope of specified type into `Scope`.
    pub fn insert<T: Any + Sync + Send>(&mut self, scope: T) -> Option<Arc<T>> {
        self.scopes
            .insert(TypeId::of::<T>(), Arc::new(scope))
            .map(|arc| arc.downcast::<T>().unwrap())
    }

    /// Removes a scope of specified type from `Scope`.
    /// This may be useful in middlewares, that change protocol identifier or in inter-middleware
    /// communication.
    pub fn remove<T: Any + Sync + Send>(&mut self) -> Option<Arc<T>> {
        self.scopes
            .remove(&TypeId::of::<T>())
            .map(|arc| arc.downcast::<T>().unwrap())
    }

    /// Consumes `self` and returns new `Scope` with scope inserted in internal map.
    pub fn with_scope<T: Any + Sync + Send>(mut self, scope: T) -> Self {
        let _ = self.insert(scope);
        self
    }
}

/// Structure for representing an event, that is sent or received over Server- and AppStreams.
/// Event contains family, that could be used for matching inside servers, apps and middlewares.
#[derive(Clone, Debug)]
pub struct Event {
    family: Cow<'static, str>,
    event: Arc<dyn Any + Sync + Send>,
}

impl Event {
    /// Creates new event of specified family and type.
    pub fn new<T: Any + Sync + Send>(family: Cow<'static, str>, event: T) -> Self {
        Self {
            family,
            event: Arc::new(event),
        }
    }

    /// Returns event family.
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Casts and returns event of concrete type.
    pub fn get<T: Any + Sync + Send>(&self) -> Option<Arc<T>> {
        self.event.clone().downcast::<T>().ok()
    }
}

/// Trait, representing
pub trait Service<ServerStream: Stream<Item = Event>> {
    type AppStream: Stream<Item = Event> + Send + Unpin;
    type Error: StdError;

    /// Main function of a service.
    fn call(
        &mut self,
        scope: Scope,
        server_events: ServerStream,
    ) -> Result<Self::AppStream, Self::Error>;
}
