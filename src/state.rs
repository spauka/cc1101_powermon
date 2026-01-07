use core::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Initialized = 0,
    Running = 1,
    Reading = 2,
    Stopping = 3,
    Stopped = 4,
}

impl State {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => State::Initialized,
            1 => State::Running,
            2 => State::Reading,
            3 => State::Stopping,
            _ => State::Stopped,
        }
    }
}

pub struct AtomicState {
    inner: AtomicU8,
}

impl AtomicState {
    #[inline]
    pub const fn new(initial: State) -> Self {
        Self {
            inner: AtomicU8::new(initial as u8),
        }
    }

    #[inline]
    pub fn load(&self) -> State {
        State::from_u8(self.inner.load(Ordering::Acquire))
    }

    #[inline]
    pub fn store(&self, next: State) {
        self.inner.store(next.as_u8(), Ordering::Release);
    }

    #[inline]
    pub fn update_if<F>(&self, predicate: F, next: State) -> State
    where
        F: Fn(State) -> bool,
    {
        let _ = self
            .inner
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                let current_state = State::from_u8(current);
                if predicate(current_state) {
                    Some(next.as_u8())
                } else {
                    None
                }
            });

        self.load()
    }
}