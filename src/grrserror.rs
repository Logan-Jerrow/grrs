use log::error;
use std::{ops::Deref, process::Termination};

pub type GrrsResult<T> = anyhow::Result<T, GrrsError>;

#[derive(Debug)]
pub struct GrrsError {
    error: anyhow::Error,
    exit_code: exitcode::ExitCode,
}

impl GrrsError {
    pub fn from_anyhow(error: anyhow::Error, exit_code: exitcode::ExitCode) -> Self {
        GrrsError { error, exit_code }
    }
}
impl std::fmt::Display for GrrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.error, self.exit_code)
    }
}

impl std::error::Error for GrrsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.error)
    }
}

// Is there a better way?
pub struct Wrap(pub GrrsResult<()>);

impl Deref for Wrap {
    type Target = GrrsResult<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Termination for Wrap {
    fn report(self) -> std::process::ExitCode {
        match &*self {
            Ok(_) => std::process::ExitCode::SUCCESS,
            Err(r) => {
                error!(target: "grrs", "{:?}", r.error);
                std::process::ExitCode::from(r.exit_code as u8)
            }
        }
    }
}
