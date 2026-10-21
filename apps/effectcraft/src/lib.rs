//! EffectCraft on Android: the app as a library (`libeffectcraft.so`) started by `android_main`
//! inside a NativeActivity. The desktop executable is `main.rs` and shares nothing with this file
//! except the modules below; its startup is unchanged.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod audio_out;
pub mod launch_guard;
pub mod storage_access;

use effectcraft_ui_egui::EffectcraftApp;

/// The window's wgpu device asks for the adapter's own limits (textures up to 16384 px)
/// instead of eframe's fixed set: an older GPU that can't meet those (Intel HD Graphics 5500,
/// #198) still opens the app, and the compositor then checks what it needs and falls back to
/// the CPU when the device can't run it. `backends`: Window Graphics' choice, when it makes one.
pub fn wgpu_options(backends: Option<eframe::wgpu::Backends>) -> eframe::WgpuConfiguration {
    use eframe::egui_wgpu::WgpuSetup;
    let mut config = eframe::WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        if let Some(b) = backends {
            setup.instance_descriptor.backends = b;
        }
        // Nothing draws or dispatches indirectly, so wgpu's check of indirect arguments only costs
        // a compute shader when the device is created, one that Metal can't compile on an Intel
        // Iris Pro: the device is lost and the window never opens.
        // `WGPU_VALIDATION_INDIRECT_CALL=1` turns the check back on.
        setup.instance_descriptor.flags = setup.instance_descriptor.flags.difference(eframe::wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL).with_env();
        setup.device_descriptor = std::sync::Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
            label: Some("EffectCraft device"),
            required_limits: device_limits(adapter.limits()),
            ..Default::default()
        });
    }
    config
}

/// The limits to request from an adapter that offers `adapter`: all of them, with textures
/// capped at 16384 px like the headless renderer's device.
pub fn device_limits(adapter: eframe::wgpu::Limits) -> eframe::wgpu::Limits {
    eframe::wgpu::Limits { max_texture_dimension_2d: adapter.max_texture_dimension_2d.min(16384), ..adapter }
}

/// The Android app: the egui app plus the launch guard's "first frame drawn" bookkeeping.
pub struct MobileApp {
    pub app: EffectcraftApp,
    pub launch: launch_guard::Launch,
    /// The first frame was drawn (logged once).
    pub drawn: bool,
}

impl eframe::App for MobileApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        if !self.drawn {
            self.drawn = true;
            log::info!("first frame drawn");
        }
        // The launch guard's marker goes once the window has kept drawing for a few seconds.
        if let Some(left) = self.launch.frame_drawn() {
            ui.ctx().request_repaint_after(left);
        }
    }

    fn on_exit(&mut self) {
        self.launch.exited();
        self.app.on_exit();
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, raw_input);
    }
}

/// Builds the app inside eframe's creation callback. Kept out of `android_main` (and not
/// Android-only) so the desktop build type-checks it too.
pub fn create_app(cc: &eframe::CreationContext<'_>) -> MobileApp {
    let gpu_failures = effectcraft_ui_egui::gpu_failure::GpuFailureBridge::new(&cc.egui_ctx);
    if let Some(rs) = &cc.wgpu_render_state {
        let errors = gpu_failures.clone();
        rs.device.on_uncaptured_error(std::sync::Arc::new(move |error| {
            let message = format!("uncaptured GPU error: {error}");
            if errors.report(&message, false) {
                log::error!("{message}");
            }
        }));
        let lost = gpu_failures.clone();
        rs.device.set_device_lost_callback(move |reason, message| {
            lost.report(&format!("GPU device lost ({reason:?}): {message}"), true);
        });
    }

    let mut session = effectcraft_host::session();
    session.check_footage_on_open = true;
    session.autosave.background = true;
    session.load_settings();

    let mut app = EffectcraftApp::new(session);
    app.set_gpu_failure_bridge(gpu_failures);
    // The touch shell (`effectcraft_ui_egui::mobile`) has its own Home screen.
    app.ui.start_screen = false;

    app.hooks.audio_device = Some(Box::new(audio_out::open));
    app.hooks.audio_devices = Some(Box::new(audio_out::devices));
    app.hooks.media_access = Some(Box::new(storage_access::media_access));
    app.hooks.window_insets = Some(Box::new(storage_access::window_insets));
    app.hooks.pick_media = Some(Box::new(storage_access::pick_media));
    app.hooks.take_picked = Some(Box::new(storage_access::take_picked));

    // The touch shell sizes everything in dp, so egui's native pixels-per-point is kept (1 point =
    // 1 dp); Settings > Appearance > UI Scale still applies.
    MobileApp { app, launch: launch_guard::Launch::begin(None, false), drawn: false }
}

/// Android entry point: the full engine with the touch UI in a NativeActivity.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: android_activity::AndroidApp) {
    // Projects and exports live in the app's own storage (no permission needed to write there).
    if let Some(dir) = app.external_data_path().or_else(|| app.internal_data_path()) {
        effectcraft_ui_egui::mobile::set_data_dir(&dir.to_string_lossy());
        storage_access::set_data_dir(&dir);
    }
    android_logger::init_once(android_logger::Config::default().with_tag("EffectCraft").with_max_level(log::LevelFilter::Info));
    effectcraft_engine::logging::install_panic_hook();
    log::info!("EffectCraft Android: starting");
    effectcraft_engine::text::fonts::scan_system_in_background();

    let options = eframe::NativeOptions { android_app: Some(app), wgpu_options: wgpu_options(None), ..Default::default() };
    let result = eframe::run_native("EffectCraft", options, Box::new(|cc| Ok(Box::new(create_app(cc)))));
    if let Err(e) = result {
        log::error!("EffectCraft Android run_native error: {e:?}");
    }
}
