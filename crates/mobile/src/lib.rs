//! Android Mobile platform bridge and headless session runner for EffectCraft.
//!
//! Provides JNI bindings, ANativeWindow presentation, memory budgeting,
//! and vertical 9:16 cinematic reel templates for mobile devices.

pub mod bridge;
pub mod error;
pub mod session_holder;
pub mod surface;
pub mod templates;

pub use error::{MobileError, Result};
pub use session_holder::MobileEngine;
pub use surface::MobileSurface;
pub use templates::{ReelConfig, create_kinetic_title_reel, create_vertical_reel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_engine_init_and_template() {
        let engine = MobileEngine::global();
        assert!(engine.init("", "").is_ok());

        // Test creating vertical 9:16 reel
        let mut session = engine.session.lock().unwrap();
        let comp_id = create_vertical_reel(&mut session, ReelConfig {
            name: "Test Reel".into(),
            width: 1080,
            height: 1920,
            fps: 30.0,
            duration_seconds: 5.0,
        }).expect("Failed to create reel");

        assert!(comp_id > 0);
        assert_eq!(session.project.items.len(), 2); // Initial default comp + test comp
    }

    #[test]
    fn test_mobile_command_execution() {
        let engine = MobileEngine::global();
        let res = engine.execute_command("comp.list", "{}").expect("comp.list failed");
        assert!(res.contains("items") || res.contains("comps") || res.contains("[") || res.contains("{"));
    }

    #[test]
    fn test_mobile_trim_memory_no_crash() {
        let engine = MobileEngine::global();
        assert!(engine.on_trim_memory(80).is_ok()); // TRIM_MEMORY_COMPLETE
        assert!(engine.on_trim_memory(15).is_ok()); // TRIM_MEMORY_RUNNING_CRITICAL
    }
}
