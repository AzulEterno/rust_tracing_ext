use std::panic::Location;
use std::sync::Mutex;

use crate::error::InitError;
static GLOBAL_INIT: Mutex<GlobalInitState> = Mutex::new(GlobalInitState::Vacant);

enum GlobalInitState {
    Vacant,
    Initializing(&'static Location<'static>),
    Initialized(&'static Location<'static>),
}

pub(super) struct GlobalInitClaim {
    site: &'static Location<'static>,
    active: bool,
}

impl GlobalInitClaim {
    pub(super) fn acquire(site: &'static Location<'static>) -> Result<Self, InitError> {
        let mut state = GLOBAL_INIT
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match *state {
            GlobalInitState::Vacant => {
                *state = GlobalInitState::Initializing(site);
                Ok(Self { site, active: true })
            }
            GlobalInitState::Initializing(first) | GlobalInitState::Initialized(first) => {
                Err(InitError::AlreadyInitialized { first })
            }
        }
    }

    pub(super) fn commit(mut self) {
        let mut state = GLOBAL_INIT
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *state = GlobalInitState::Initialized(self.site);
        self.active = false;
    }
}

impl Drop for GlobalInitClaim {
    fn drop(&mut self) {
        if self.active {
            let mut state = GLOBAL_INIT
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            *state = GlobalInitState::Vacant;
        }
    }
}
