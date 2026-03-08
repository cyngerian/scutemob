/// Timeout for LSP requests in seconds.
/// Large workspaces (200k+ LOC) can take 60s+ for initial hover/definition requests.
pub const LSP_REQUEST_TIMEOUT_SECS: u64 = 90;

/// Delay after opening a document to allow rust-analyzer to process it.
pub const DOCUMENT_OPEN_DELAY_MILLIS: u64 = 200;
