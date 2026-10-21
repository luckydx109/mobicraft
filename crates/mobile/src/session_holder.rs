//! Global state holder and lifecycle manager for EffectCraft Mobile.
//!
//! Provides synchronized access to the engine `Session`, `MobileSurface`,
//! background export progress, and low-memory handlers.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use effectcraft_engine::Session;
use effectcraft_project::ItemId;
use effectcraft_render::{NoFootage, RenderOpts, Renderer};
use effectcraft_time::Tick;
use serde_json::{Value, json};

use crate::error::{MobileError, Result};
use crate::surface::MobileSurface;

pub struct MobileEngine {
    pub session: Mutex<Session>,
    pub surface: Mutex<MobileSurface>,
    pub active_comp: Mutex<Option<ItemId>>,
    pub preview_scale: Mutex<f64>,
    pub is_exporting: AtomicBool,
    pub cancel_export: AtomicBool,
    pub export_progress: AtomicU32, // 0 to 10000 -> 0.00% to 100.00%
}

static ENGINE_INSTANCE: OnceLock<Arc<MobileEngine>> = OnceLock::new();

impl MobileEngine {
    /// Retrieve or initialize the singleton MobileEngine instance.
    pub fn global() -> Arc<MobileEngine> {
        ENGINE_INSTANCE
            .get_or_init(|| {
                Arc::new(MobileEngine {
                    session: Mutex::new(Session::new()),
                    surface: Mutex::new(MobileSurface::new()),
                    active_comp: Mutex::new(None),
                    preview_scale: Mutex::new(0.5), // 0.5x default preview scale on mobile
                    is_exporting: AtomicBool::new(false),
                    cancel_export: AtomicBool::new(false),
                    export_progress: AtomicU32::new(0),
                })
            })
            .clone()
    }

    /// Initialize directories and configuration.
    pub fn init(&self, cache_dir: &str, _internal_dir: &str) -> Result<()> {
        let mut session = self.session.lock().map_err(|_| MobileError::Engine(effectcraft_engine::EngineError::LockPoisoned))?;
        
        // Configure disk cache folder if provided
        if !cache_dir.is_empty() {
            session.prefs.media.disk_cache_folder = cache_dir.to_string();
            session.prefs.media.disk_cache_enabled = true;
        }

        // Setup default 9:16 vertical composition for mobile reels
        let _ = crate::templates::create_vertical_reel(&mut session, crate::templates::ReelConfig::default());
        if let Some(comp_id) = session.state.active_comp {
            if let Ok(mut ac) = self.active_comp.lock() {
                *ac = Some(comp_id);
            }
        }

        log::info!("EffectCraft Mobile Engine initialized with cache_dir: {}", cache_dir);
        Ok(())
    }

    /// Execute a command by name with JSON parameters and return the JSON result.
    pub fn execute_command(&self, command_id: &str, params_json: &str) -> Result<String> {
        let params: Value = if params_json.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(params_json)?
        };

        let mut session = self.session.lock().map_err(|_| MobileError::Engine(effectcraft_engine::EngineError::LockPoisoned))?;
        let result = session.execute(command_id, params)?;
        
        // Sync active comp tracking if changed
        if let Some(comp_id) = session.state.active_comp {
            if let Ok(mut ac) = self.active_comp.lock() {
                *ac = Some(comp_id);
            }
        }

        Ok(serde_json::to_string(&result)?)
    }

    /// Render a frame at `tick` with the requested preview scale, and present it to the attached surface.
    pub fn seek_frame(&self, comp_id: Option<u64>, tick: i64, scale: Option<f64>) -> Result<()> {
        let session = self.session.lock().map_err(|_| MobileError::Engine(effectcraft_engine::EngineError::LockPoisoned))?;
        
        let cid = comp_id
            .map(ItemId)
            .or_else(|| self.active_comp.lock().ok().and_then(|c| *c))
            .or(session.state.active_comp)
            .ok_or_else(|| MobileError::InvalidParam("No active composition available to render".into()))?;

        let render_scale = scale.unwrap_or_else(|| {
            self.preview_scale.lock().ok().map(|s| *s).unwrap_or(0.5)
        }).clamp(0.1, 1.0);

        let opts = RenderOpts {
            scale: render_scale,
            motion_blur: false, // Suppress multi-sample subframe motion blur during scrubbing
            backend: effectcraft_render::Backend::Auto,
            ..Default::default()
        };

        // Render frame with footage source
        let footage = session.footage.as_ref();
        let mut renderer = Renderer::new(&session.project, footage, opts);
        renderer.cache = Some(&session.layer_cache);
        
        let target_tick = Tick(tick);
        let frame = renderer.comp_frame(cid, target_tick);

        // Present to surface
        let mut surface = self.surface.lock().map_err(|_| MobileError::Surface("Surface lock poisoned".into()))?;
        surface.present(&frame)?;

        Ok(())
    }

    /// Forward Android memory trim event to the engine.
    pub fn on_trim_memory(&self, level: i32) -> Result<()> {
        let mut session = self.session.lock().map_err(|_| MobileError::Engine(effectcraft_engine::EngineError::LockPoisoned))?;
        session.on_trim_memory(level);
        Ok(())
    }

    /// Cancel a running export job.
    pub fn cancel_export(&self) {
        self.cancel_export.store(true, Ordering::SeqCst);
    }

    /// Get the current export progress (0.0 to 1.0).
    pub fn get_export_progress(&self) -> f32 {
        let raw = self.export_progress.load(Ordering::Relaxed);
        (raw as f32) / 10000.0
    }

    /// Set the export progress (0.0 to 1.0).
    pub fn set_export_progress(&self, progress: f32) {
        let val = (progress.clamp(0.0, 1.0) * 10000.0) as u32;
        self.export_progress.store(val, Ordering::Relaxed);
    }
}
