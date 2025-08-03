use log::{error, warn};

pub struct FatalError;

impl std::fmt::Debug for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered a fatal error, cannot continue")
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for FatalError {}

pub trait Log {
    type FatalSuccess;
    fn warn(self) -> Self;
    fn error(self) -> Self;
    fn fatal(self) -> Result<Self::FatalSuccess, FatalError>;
}

impl<T> Log for Result<T, anyhow::Error> {
    type FatalSuccess = T;
    #[inline(always)]
    fn warn(self) -> Self {
        self.inspect_err(|err| warn!("{err:?}"))
    }
    #[inline(always)]
    fn error(self) -> Self {
        self.inspect_err(|err| error!("{err:?}"))
    }
    #[inline(always)]
    fn fatal(self) -> Result<Self::FatalSuccess, FatalError> {
        self.map_err(|err| {
            error!(target: "fatal", "{err:?}");
            FatalError
        })
    }
}
