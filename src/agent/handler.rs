use crate::agent::State;

/// Closures based event handler.
///
/// Any closure from given handler can be invoked in any thread, usually from dedicated internal
/// libjuice thread.
///
/// # Example
/// ```
/// # use libjuice_rs::Handler;
/// let h: Handler = Handler::default()
///     .state_handler(|s| println!("State changed to: {:?}", s))
///     .candidate_handler(|c| println!("Local candidate: {:?}", c))
///     .gathering_done_handler(||println!("Gathering done!"))
///     .recv_handler(|packet| println!("Received packet of length: {}", packet.len()));
/// ```
#[derive(Default)]
pub struct Handler {
    /// ICE state change handler
    on_state_change: Option<Box<dyn FnMut(State) + Send + Sync + 'static>>,
    /// Local ICE candidate handler
    on_candidate: Option<Box<dyn FnMut(String) + Send + Sync + 'static>>,
    /// Gathering stage finish handler
    on_gathering_done: Option<Box<dyn FnMut() + Send + Sync + 'static>>,
    /// Incoming packet
    on_recv: Option<Box<dyn FnMut(&[u8]) + Send + Sync + 'static>>,
}

impl Handler {
    /// Set ICE state change handler
    pub fn state_handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(State),
        F: Send + Sync + 'static,
    {
        self.on_state_change = Some(Box::new(f));
        self
    }

    /// Set local candidate handler
    pub fn candidate_handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(String),
        F: Send + Sync + 'static,
    {
        self.on_candidate = Some(Box::new(f));
        self
    }

    /// Set gathering done handler
    pub fn gathering_done_handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(),
        F: Send + Sync + 'static,
    {
        self.on_gathering_done = Some(Box::new(f));
        self
    }

    /// Set incoming packet handler
    pub fn recv_handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(&[u8]),
        F: Send + Sync + 'static,
    {
        self.on_recv = Some(Box::new(f));
        self
    }

    pub(crate) fn on_state_changed(&mut self, state: State) {
        if let Some(f) = &mut self.on_state_change {
            f(state)
        }
    }

    pub(crate) fn on_candidate(&mut self, candidate: String) {
        if let Some(f) = &mut self.on_candidate {
            f(candidate)
        }
    }

    pub(crate) fn on_gathering_done(&mut self) {
        if let Some(f) = &mut self.on_gathering_done {
            f()
        }
    }

    pub(crate) fn on_recv(&mut self, packet: &[u8]) {
        if let Some(f) = &mut self.on_recv {
            f(packet)
        }
    }
}
