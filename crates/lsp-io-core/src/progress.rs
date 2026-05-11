use std::path::PathBuf;

/// Domain progress emitted by long-running install and removal operations.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    ResolveStart { servers: Vec<String> },
    InstallStart { server: String, version: String },
    InstallOutput { server: String, line: String },
    InstallComplete { server: String, path: PathBuf },
    InstallFailed { server: String, error: String },
    RemovalComplete { server: String, message: String },
}

pub trait ProgressHandler: Send + Sync {
    fn on_event(&self, event: ProgressEvent);
}

pub struct NoopProgress;

impl ProgressHandler for NoopProgress {
    fn on_event(&self, _event: ProgressEvent) {}
}
