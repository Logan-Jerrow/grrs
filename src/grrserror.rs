use exitcode::ExitCode;

#[derive(Debug)]
pub struct GrrsError {
    pub error: anyhow::Error,
    pub exit_code: ExitCode,
}

impl GrrsError {
    pub fn from_anyhow(error: anyhow::Error, exit_code: ExitCode) -> Self {
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
