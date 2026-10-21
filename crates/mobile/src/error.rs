//! Error handling for EffectCraft Mobile.
//!
//! Every operation returns a `Result<T, MobileError>` and NEVER panics.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MobileError {
    #[error("Engine error: {0}")]
    Engine(#[from] effectcraft_engine::EngineError),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Export error: {0}")]
    Export(#[from] effectcraft_export::ExportError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JNI error: {0}")]
    Jni(String),

    #[error("Surface error: {0}")]
    Surface(String),

    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    #[error("Session not initialized")]
    NotInitialized,
}

pub type Result<T> = std::result::Result<T, MobileError>;
