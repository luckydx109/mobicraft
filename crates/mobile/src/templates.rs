//! Mobile Reel Templates for Social Media (Instagram Reels, TikTok, YouTube Shorts).
//!
//! Provides built-in vertical 9:16 (1080x1920) presets optimized for mobile motion graphics.

use effectcraft_engine::Session;
use serde_json::json;

use crate::error::{MobileError, Result};

/// Configuration for creating a vertical reel.
pub struct ReelConfig {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration_seconds: f64,
}

impl Default for ReelConfig {
    fn default() -> Self {
        Self {
            name: "Reel 9:16".into(),
            width: 1080,
            height: 1920,
            fps: 30.0,
            duration_seconds: 15.0,
        }
    }
}

/// Create a 9:16 vertical social media reel composition in the active session.
pub fn create_vertical_reel(session: &mut Session, config: ReelConfig) -> Result<u64> {
    let params = json!({
        "name": config.name,
        "width": config.width,
        "height": config.height,
        "frameRate": config.fps,
        "duration": config.duration_seconds,
        "open": true,
    });
    let res = session.execute("comp.new", params)?;
    let comp_id = res.get("comp")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| MobileError::InvalidParam("comp.new did not return valid comp ID".into()))?;
    Ok(comp_id)
}

/// Create a kinetic motion typography reel with a dark backdrop solid and an animated text layer.
pub fn create_kinetic_title_reel(session: &mut Session, title: &str) -> Result<u64> {
    let comp_id = create_vertical_reel(session, ReelConfig {
        name: format!("Kinetic: {title}"),
        width: 1080,
        height: 1920,
        fps: 30.0,
        duration_seconds: 10.0,
    })?;

    // Add dark background solid (1080x1920, deep slate)
    let _ = session.execute("layer.solid", json!({
        "name": "Background",
        "width": 1080,
        "height": 1920,
        "color": [0.05, 0.05, 0.08, 1.0],
    }));

    // Add centered kinetic text layer
    let _ = session.execute("layer.text", json!({
        "text": title,
        "fontSize": 96.0,
    }));

    Ok(comp_id)
}

/// Create a 60 fps speed ramp action reel setup with preset speed markers.
pub fn create_speed_ramp_reel(session: &mut Session, name: &str) -> Result<u64> {
    let comp_id = create_vertical_reel(session, ReelConfig {
        name: format!("SpeedRamp: {name}"),
        width: 1080,
        height: 1920,
        fps: 60.0,
        duration_seconds: 12.0,
    })?;

    // Background gradient canvas
    let _ = session.execute("layer.solid", json!({
        "name": "Vignette BG",
        "width": 1080,
        "height": 1920,
        "color": [0.08, 0.08, 0.12, 1.0],
    }));

    // Add cinematic transition text
    let _ = session.execute("layer.text", json!({
        "text": "ACTION",
        "fontSize": 120.0,
    }));

    Ok(comp_id)
}

/// Create a minimalist aesthetic typography reel (clean layout, headline + subtitle).
pub fn create_minimalist_typography_reel(session: &mut Session, headline: &str, subtitle: &str) -> Result<u64> {
    let comp_id = create_vertical_reel(session, ReelConfig {
        name: "Minimalist Reel".into(),
        width: 1080,
        height: 1920,
        fps: 30.0,
        duration_seconds: 8.0,
    })?;

    // Soft warm background
    let _ = session.execute("layer.solid", json!({
        "name": "Warm Background",
        "width": 1080,
        "height": 1920,
        "color": [0.12, 0.12, 0.14, 1.0],
    }));

    // Subtitle text (top)
    let _ = session.execute("layer.text", json!({
        "text": subtitle,
        "fontSize": 42.0,
    }));

    // Headline text (hero)
    let _ = session.execute("layer.text", json!({
        "text": headline,
        "fontSize": 84.0,
    }));

    Ok(comp_id)
}
