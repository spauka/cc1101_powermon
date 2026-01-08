use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramState {
    Initialized = 0,
    Running = 1,
    Reading = 2,
    Stopping = 3,
    Stopped = 4,
}

impl From<u8> for ProgramState {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => ProgramState::Initialized,
            1 => ProgramState::Running,
            2 => ProgramState::Reading,
            3 => ProgramState::Stopping,
            _ => ProgramState::Stopped,
        }
    }
}

impl From<ProgramState> for u8 {
    #[inline]
    fn from(s: ProgramState) -> Self {
        s as u8
    }
}

pub struct AtomicState<T: Copy + Into<u8> + From<u8>> {
    inner: AtomicU8,
    _marker: PhantomData<T>,
}

impl<T: Copy + Into<u8> + From<u8>> AtomicState<T> {
    #[inline]
    pub fn new(initial: T) -> Self {
        Self {
            inner: AtomicU8::new(initial.into()),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn load(&self) -> T {
        T::from(self.inner.load(Ordering::Acquire))
    }

    #[inline]
    pub fn store(&self, next: T) {
        self.inner.store(next.into(), Ordering::Release);
    }

    #[inline]
    pub fn update_if<F>(&self, predicate: F, next: T) -> Result<T, T>
    where
        F: Fn(T) -> bool,
    {
        let res = self
            .inner
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                let current_state: T = T::from(current);
                if predicate(current_state) {
                    Some(next.into())
                } else {
                    None
                }
            });

        match res {
            Ok(r) => Ok(T::from(r)),
            Err(r) => Err(T::from(r))
        }
    }
}