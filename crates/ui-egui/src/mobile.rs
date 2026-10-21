//! Touch-first phone UI: the same engine, commands and panels as the desktop UI, arranged for
//! thumbs. The editor follows the EffectCraft Mobile reference: a thin toolbar (Home, Import,
//! Options, Undo/Redo, Export) with the tool strip under it, the composition preview with its
//! scrolling control strip, a composition-navigation strip, and the timeline filling the rest,
//! with a floating + that swaps the timeline for a section panel (Assets, Effects, …).
//!
//! Active on Android, or on any platform when `EFFECTCRAFT_MOBILE_UI=1` is set (to try it in a
//! narrow desktop window). Everything opens closed: no menu, popup, sheet or extra panel is shown
//! until the user asks for it. Nothing here edits the project directly; every edit is a command
//! (`menus::invoke` / `Session::execute`), as in the desktop UI.
//!
//! The mobile UI lives in the `mobile*.rs` modules; the shared files carry only small, marked
//! hooks (see `docs/MOBILE_UI.md`), so updates from the desktop repository merge cleanly.

use egui::{Align2, Color32, Id, Rect, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::dock::PanelKind;
use crate::icons::Icon;
use crate::mobile_kit::{self as kit, Btn, Item};
use crate::state::Tool;
use crate::theme::Tokens;
use crate::{EffectcraftApp, menus, panels};

pub use crate::mobile_kit::dp;

static FORCED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Switch the phone layout on regardless of platform (tests, previews).
pub fn set_forced(on: bool) {
    FORCED.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether the phone layout replaces the docked desktop layout.
pub fn active() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    cfg!(target_os = "android") || FORCED.load(std::sync::atomic::Ordering::Relaxed) || *ENV.get_or_init(|| std::env::var_os("EFFECTCRAFT_MOBILE_UI").is_some_and(|v| v != "0"))
}

/// Extra room (points) the bottom-left toast needs (none: the editor has no bottom rail).
pub fn toast_lift(_ctx: &egui::Context) -> f32 {
    0.0
}

/// Called by the timeline when a layer is double-tapped: open its action sheet.
pub fn open_layer_menu(ctx: &egui::Context, layer: u64) {
    ctx.data_mut(|d| d.insert_temp(Id::new("m-layer-menu"), layer));
    ctx.request_repaint();
}

static DATA_DIR: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Set where projects and exports live (the Android entry point passes the app's storage).
pub fn set_data_dir(dir: &str) {
    if let Ok(mut d) = DATA_DIR.write() {
        *d = Some(dir.to_string());
    }
}

/// Where projects and exports live: the directory given to [`set_data_dir`], else
/// `EFFECTCRAFT_DATA_DIR`, else `~/EffectCraft`.
pub fn data_dir() -> String {
    DATA_DIR
        .read()
        .ok()
        .and_then(|d| d.clone())
        .or_else(|| std::env::var("EFFECTCRAFT_DATA_DIR").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| format!("{}/EffectCraft", std::env::var("HOME").unwrap_or_else(|_| ".".into())))
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Assets,
    Effects,
    Settings,
    Preview,
    Properties,
    Align,
    Audio,
    RenderQueue,
    EssentialGraphics,
    Paint,
    Other(PanelKind),
}

impl Section {
    fn panel(self) -> PanelKind {
        match self {
            Section::Assets => PanelKind::Project,
            Section::Effects => PanelKind::EffectsPresets,
            Section::Settings => PanelKind::EffectControls,
            Section::Preview => PanelKind::Preview,
            Section::Properties => PanelKind::Properties,
            Section::Align => PanelKind::Align,
            Section::Audio => PanelKind::Audio,
            Section::RenderQueue => PanelKind::RenderQueue,
            Section::EssentialGraphics => PanelKind::EssentialGraphics,
            Section::Paint => PanelKind::Paint,
            Section::Other(p) => p,
        }
    }
}

const SECTIONS: [(Section, &str); 16] = [
    (Section::Assets, "Assets"),
    (Section::Effects, "Effects & Presets"),
    (Section::Settings, "Effect Settings"),
    (Section::Preview, "Preview"),
    (Section::Properties, "Properties"),
    (Section::Align, "Align"),
    (Section::Audio, "Audio"),
    (Section::RenderQueue, "Render Queue"),
    (Section::EssentialGraphics, "Essential Graphics"),
    (Section::Paint, "Paint & Brushes"),
    (Section::Other(PanelKind::Character), "Character"),
    (Section::Other(PanelKind::Paragraph), "Paragraph"),
    (Section::Other(PanelKind::History), "History"),
    (Section::Other(PanelKind::Markers), "Markers"),
    (Section::Other(PanelKind::Info), "Info"),
    (Section::Other(PanelKind::Flowchart), "Flowchart"),
];

/// Pages that cover the editor.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Page {
    Editor,
    Import,
    Export,
    NewComp,
    ConfirmHome,
}

#[derive(Clone, Copy)]
struct MState {
    /// The tool strip under the toolbar.
    tools_row: bool,
    /// The Options (File … Help) strip.
    menu_row: bool,
    /// The open menu popup: which menu and where it hangs from.
    menu: Option<(usize, Rect)>,
    /// The sub-tool popup: which toolbar slot and where it hangs from.
    tool_pop: Option<(usize, Rect)>,
    /// The + panel's section (`None` = the timeline).
    plus: Option<Section>,
    /// Preview's share of the free height (width in landscape).
    split: f32,
    page: Page,
    /// How far the whole timeline (layer names, properties and the time graph) is slid sideways.
    tl_pan: f32,
    /// Where the floating + was dragged to, as fractions of the timeline area (`None`: its corner).
    plus_pos: Option<egui::Vec2>,
    /// Whether the column configuration sheet is open.
    column_sheet: bool,
}

impl Default for MState {
    fn default() -> Self {
        MState {
            tools_row: true,
            menu_row: false,
            menu: None,
            tool_pop: None,
            plus: None,
            split: 0.42,
            page: Page::Editor,
            tl_pan: 0.0,
            plus_pos: None,
            column_sheet: false,
        }
    }
}

fn load(ctx: &egui::Context) -> MState {
    ctx.data(|d| d.get_temp::<MState>(Id::new("mobile-state"))).unwrap_or_default()
}

fn store(ctx: &egui::Context, s: MState) {
    ctx.data_mut(|d| d.insert_temp(Id::new("mobile-state"), s));
}

/// Open one of the pages that cover the editor.
pub(crate) fn open_page(ctx: &egui::Context, page: Page) {
    let mut st = load(ctx);
    st.page = page;
    store(ctx, st);
}

fn run(app: &mut EffectcraftApp, ctx: &egui::Context, id: &str) {
    if let Err(e) = menus::invoke(app, ctx, id, json!({})) {
        app.ui.status = e;
    }
}

/// Touch sizes for the desktop panels that are embedded as they are: taller rows, taller
/// fields, a wider grab radius. Idempotent, so it can run every frame (preference changes
/// rebuild the tokens and the style).
fn touch_style(app: &mut EffectcraftApp, ctx: &egui::Context, u: f32) {
    let row = (32.0 * u).round();
    if app.tokens.row_h < row {
        app.tokens.row_h = row;
    }
    let tab = (40.0 * u).round();
    if app.tokens.tab_h < tab {
        app.tokens.tab_h = tab;
    }
    let h = (36.0 * u).round();
    if ctx.global_style().spacing.interact_size.y < h {
        ctx.global_style_mut(|s| {
            s.spacing.interact_size.y = h;
            s.spacing.button_padding = vec2(12.0 * u, 6.0 * u);
            s.spacing.item_spacing = vec2(8.0 * u, 6.0 * u);
            s.spacing.icon_width = 22.0 * u;
            s.spacing.slider_width = 160.0 * u;
            s.interaction.interact_radius = 10.0 * u;
        });
    }
}

fn text_w(ui: &Ui, s: &str, u: f32) -> f32 {
    ui.painter().layout_no_wrap(s.to_string(), Tokens::medium(13.5), Color32::WHITE).size().x + 28.0 * u
}

/// A toolbar slot with the tools that share it (tap again to pick another, as in After Effects).
struct Group {
    name: &'static str,
    tools: &'static [(Tool, &'static str)],
    needs_3d: bool,
}

const GROUPS: [Group; 8] = [
    Group { name: "Selection", tools: &[(Tool::Selection, "Selection")], needs_3d: false },
    Group { name: "Hand", tools: &[(Tool::Hand, "Hand"), (Tool::Zoom, "Zoom")], needs_3d: false },
    Group { name: "Rotation", tools: &[(Tool::Rotate, "Rotation")], needs_3d: false },
    Group { name: "Anchor", tools: &[(Tool::PanBehind, "Pan Behind (Anchor Point)")], needs_3d: false },
    Group {
        name: "Shape",
        tools: &[(Tool::Rectangle, "Rectangle"), (Tool::RoundedRect, "Rounded Rectangle"), (Tool::Ellipse, "Ellipse"), (Tool::Polygon, "Polygon"), (Tool::Star, "Star")],
        needs_3d: false,
    },
    Group {
        name: "Pen",
        tools: &[(Tool::Pen, "Pen"), (Tool::PenAdd, "Add Vertex"), (Tool::PenDelete, "Delete Vertex"), (Tool::PenConvert, "Convert Vertex"), (Tool::MaskFeather, "Mask Feather")],
        needs_3d: false,
    },
    Group { name: "Text", tools: &[(Tool::Type, "Horizontal Type"), (Tool::TypeVertical, "Vertical Type")], needs_3d: false },
    Group { name: "Camera", tools: &[(Tool::Orbit, "Orbit Camera"), (Tool::PanCamera, "Pan Camera"), (Tool::Dolly, "Dolly Camera")], needs_3d: true },
];

fn last_tool(ctx: &egui::Context, g: usize) -> Tool {
    ctx.data(|d| d.get_temp::<Tool>(Id::new(("m-tool-last", g)))).filter(|t| GROUPS[g].tools.iter().any(|(x, _)| x == t)).unwrap_or(GROUPS[g].tools[0].0)
}

// ---------------------------------------------------------------------------------------------
// The frame
// ---------------------------------------------------------------------------------------------

/// After the system gallery / file manager was opened: wait for what the user picked (copied into
/// the app's storage by the host), then import it, into the open composition when there is one.
fn poll_picked(app: &mut EffectcraftApp, ctx: &egui::Context) {
    let pending = Id::new("mobile-pick-pending");
    if !ctx.data(|d| d.get_temp::<bool>(pending)).unwrap_or(false) {
        return;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(300));
    let Some(take) = app.hooks.take_picked.as_ref() else { return };
    let Some(paths) = take() else { return };
    ctx.data_mut(|d| d.insert_temp(pending, false));
    if paths.is_empty() {
        return; // cancelled
    }
    let add = app.session.active_comp_id().is_some();
    match app.session.execute("mediaBrowser.import", json!({"paths": paths, "addToComp": add})) {
        Ok(v) => {
            let n = v.get("items").and_then(|i| i.as_array()).map_or(paths.len(), |i| i.len());
            app.ui.status = if add { format!("Added {n} item(s) to the composition") } else { format!("Imported {n} item(s) to Project") };
        }
        Err(e) => app.ui.status = e.to_string(),
    }
}

/// The part of `full` that the system bars, a display cutout and the keyboard leave free (Android;
/// the host reports their size in pixels through `Hooks::window_insets`, read twice a second).
fn safe_rect(app: &EffectcraftApp, ctx: &egui::Context, full: Rect) -> Rect {
    let id = Id::new("mobile-insets");
    let now = ctx.input(|i| i.time);
    let (at, mut v): (f64, [f32; 4]) = ctx.data(|d| d.get_temp(id)).unwrap_or((f64::NEG_INFINITY, [0.0; 4]));
    if now - at > 0.4 {
        if let Some(read) = app.hooks.window_insets.as_ref() {
            let ppp = ctx.pixels_per_point().max(0.1);
            v = match read() {
                Some(px) => px.map(|n| n / ppp),
                // Not known yet (before the window's first layout): leave room for a navigation bar
                // rather than put buttons under it.
                None => [0.0, 0.0, 0.0, 28.0],
            };
        }
        ctx.data_mut(|d| d.insert_temp(id, (now, v)));
    }
    let free = Rect::from_min_max(full.min + vec2(v[0], v[1]), full.max - vec2(v[2], v[3]));
    if free.width() < 100.0 || free.height() < 100.0 { full } else { free }
}

/// Draw the whole phone UI inside `full` (replaces the menu bar, header and dock area).
pub fn show(app: &mut EffectcraftApp, ui: &mut Ui, full: Rect) {
    let ctx = ui.ctx().clone();
    // The app draws behind the system bars (status bar, navigation bar / gesture area, notch):
    // everything below is laid out in what they leave free, so no button sits under them.
    let full = safe_rect(app, &ctx, full);
    poll_picked(app, &ctx);
    let u = dp(&ctx);
    touch_style(app, &ctx, u);
    // Once: the timeline keeps only the eye/lock column and the label, so layer names and the time
    // graph both fit a phone. (The other columns stay in the column menu.)
    if !ctx.data(|d| d.get_temp::<bool>(Id::new("mobile-columns"))).unwrap_or(false) {
        for id in ["keys", "num", "comment", "switches", "modes", "parent", "in", "out", "duration", "stretch"] {
            let _ = panels::timeline::set_column(&mut app.ui.timeline, id, false);
        }
        ctx.data_mut(|d| d.insert_temp(Id::new("mobile-columns"), true));
    }
    let t = app.tokens;
    if app.ui.start_screen {
        crate::mobile_home::show(app, ui, &ctx, full, u);
        return;
    }
    let mut st = load(&ctx);
    let landscape = full.width() > full.height() * 1.15;

    // --- Rectangles ---
    let row1_h = 52.0 * u;
    let tools_h = if st.tools_row { 46.0 * u } else { 0.0 };
    let menu_h = if st.menu_row { 44.0 * u } else { 0.0 };
    let row1 = Rect::from_min_size(full.min, vec2(full.width(), row1_h));
    let tools = Rect::from_min_size(pos2(full.min.x, row1.max.y), vec2(full.width(), tools_h));
    let menu = Rect::from_min_size(pos2(full.min.x, tools.max.y), vec2(full.width(), menu_h));
    let body = Rect::from_min_max(pos2(full.min.x, menu.max.y), full.max);
    let viewbar_h = 44.0 * u;
    let nav_h = 40.0 * u;
    let handle_h = 14.0 * u;
    let (preview, viewbar, nav, handle, tl);
    if landscape {
        let left_w = (body.width() * st.split).clamp(body.width() * 0.3, body.width() * 0.65);
        let free = (body.height() - viewbar_h - nav_h).max(0.0);
        preview = Rect::from_min_size(body.min, vec2(left_w, free));
        viewbar = Rect::from_min_size(pos2(body.min.x, preview.max.y), vec2(left_w, viewbar_h));
        nav = Rect::from_min_size(pos2(body.min.x, viewbar.max.y), vec2(left_w, nav_h));
        handle = Rect::from_min_size(pos2(body.min.x + left_w - handle_h / 2.0, body.min.y), vec2(handle_h, body.height()));
        tl = Rect::from_min_max(pos2(body.min.x + left_w, body.min.y), body.max);
    } else {
        let free = (body.height() - viewbar_h - nav_h - handle_h).max(0.0);
        let pv_h = (free * st.split).clamp(free * 0.22, free * 0.7);
        preview = Rect::from_min_size(body.min, vec2(body.width(), pv_h));
        viewbar = Rect::from_min_size(pos2(body.min.x, preview.max.y), vec2(body.width(), viewbar_h));
        nav = Rect::from_min_size(pos2(body.min.x, viewbar.max.y), vec2(body.width(), nav_h));
        handle = Rect::from_min_size(pos2(body.min.x, nav.max.y), vec2(body.width(), handle_h));
        tl = Rect::from_min_max(pos2(body.min.x, handle.max.y), body.max);
    }

    // --- Parts ---
    top_bar(app, ui, &ctx, row1, &mut st, u);
    if st.tools_row {
        tool_strip(app, ui, &ctx, tools, &mut st, u);
    }
    if st.menu_row {
        menu_strip(app, ui, menu, &mut st, u);
    }
    preview_area(app, ui, &ctx, preview, u);
    crate::mobile_viewbar::show(app, ui, viewbar, full, u);
    comp_nav(app, ui, &ctx, nav, u);
    splitter(ui, handle, &mut st, landscape, body, u, &t);
    match st.plus {
        None => timeline_area(app, ui, tl, &mut st, u),
        Some(sec) => plus_panel(app, ui, tl, &mut st, sec, u),
    }
    app.dock_rects = vec![(PanelKind::Composition, preview), (st.plus.map_or(PanelKind::Timeline, Section::panel), tl)];

    // --- Menus, sheets and pages (all closed until asked for) ---
    tool_popup(app, &ctx, full, &mut st, u);
    menu_popup(app, &ctx, full, &mut st, u);
    if let Some(layer) = ctx.data(|d| d.get_temp::<u64>(Id::new("m-layer-menu"))) {
        let _ = app.session.execute("layer.select", json!({"layers": [layer]}));
        if crate::mobile_menu::layer_sheet(app, &ctx, full, layer, u) {
            ctx.data_mut(|d| d.remove::<u64>(Id::new("m-layer-menu")));
        }
    }
    if st.column_sheet {
        if column_sheet(app, &ctx, full, u) {
            st.column_sheet = false;
        }
    }
    pages(app, &ctx, full, &mut st, u);
    store(&ctx, st);
}

fn pages(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, st: &mut MState, u: f32) {
    match st.page {
        Page::Editor => {}
        Page::Import => {
            if crate::mobile_import::show(app, ctx, full, u) {
                st.page = Page::Editor;
            }
        }
        Page::Export => {
            if crate::mobile_export::show(app, ctx, full, u) {
                st.page = Page::Editor;
            }
        }
        Page::NewComp => {
            if crate::mobile_home::new_comp_form(app, ctx, full, u, false).is_some() {
                st.page = Page::Editor;
            }
        }
        Page::ConfirmHome => match crate::mobile_home::confirm_leave(app, ctx, full, u) {
            Some(true) => {
                st.page = Page::Editor;
                app.ui.start_screen = true;
            }
            Some(false) => {
                crate::mobile_home::goto(ctx, 0);
                st.page = Page::Editor;
            }
            None => {}
        },
    }
}

// ---------------------------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------------------------

fn top_bar(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, rect: Rect, st: &mut MState, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.header_bg);
    let b = 44.0 * u;
    let cy = rect.center().y;
    let cell = |x: f32| Rect::from_min_size(pos2(x, cy - b / 2.0), vec2(b, b));
    let mut x = rect.min.x + 4.0 * u;
    // Toolbar toggle, Home, Import, Options.
    if kit::icon_tap(app, ui, cell(x), Icon::ChevronDown, "Toolbar", st.tools_row, true, "mobile.toolbar", u) {
        st.tools_row = !st.tools_row;
    }
    x += b;
    if kit::icon_tap(app, ui, cell(x), Icon::Home, "Home", false, true, "mobile.home", u) {
        if app.session.is_dirty() {
            st.page = Page::ConfirmHome;
        } else {
            app.ui.start_screen = true;
        }
    }
    x += b;
    if kit::icon_tap(app, ui, cell(x), Icon::Plus, "Import", false, true, "mobile.import", u) {
        st.page = Page::Import;
    }
    x += b;
    if kit::icon_tap(app, ui, cell(x), Icon::Hamburger, "Options", st.menu_row, true, "mobile.options", u) {
        st.menu_row = !st.menu_row;
        st.menu = None;
    }
    // Export, Redo, Undo from the right.
    let ew = if rect.width() < 360.0 * u { 66.0 * u } else { 84.0 * u };
    let mut rx = rect.max.x - 8.0 * u;
    let export = Rect::from_min_size(pos2(rx - ew, cy - 18.0 * u), vec2(ew, 36.0 * u));
    if kit::button(app, ui, export, "Export", Btn::Primary, true, "mobile.export", u) {
        st.page = Page::Export;
    }
    rx -= ew + 4.0 * u;
    let can_redo = !app.session.history.redo.is_empty();
    if kit::icon_tap(app, ui, cell(rx - b), Icon::Redo, "Redo", false, can_redo, "mobile.redo", u) {
        run(app, ctx, "edit.redo");
    }
    rx -= b;
    let can_undo = !app.session.history.undo.is_empty();
    if kit::icon_tap(app, ui, cell(rx - b), Icon::Undo, "Undo", false, can_undo, "mobile.undo", u) {
        run(app, ctx, "edit.undo");
    }
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.app_bg));
}

fn tool_strip(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, rect: Rect, st: &mut MState, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.app_bg));
    let has_3d = app.session.active_comp().is_some_and(|c| c.has_3d());
    let w = (rect.width() / GROUPS.len() as f32).max(48.0 * u);
    let items: Vec<Item> = GROUPS
        .iter()
        .enumerate()
        .map(|(g, grp)| {
            let cur = if grp.tools.iter().any(|(x, _)| *x == app.ui.tool) { app.ui.tool } else { last_tool(ctx, g) };
            Item::icon(&format!("mobile.tool.{}", grp.name), cur.icon(), w).on(grp.tools.iter().any(|(x, _)| *x == app.ui.tool)).enabled(!grp.needs_3d || has_3d)
        })
        .collect();
    let out = kit::strip(app, ui, rect, "tools", &items, u);
    // A small corner mark on slots that hold more tools.
    for (g, r) in out.rects.iter().enumerate() {
        if GROUPS[g].tools.len() > 1 {
            let c = pos2(r.max.x - 9.0 * u, r.max.y - 9.0 * u);
            let col = if items[g].on { Color32::WHITE } else { t.text_dim };
            ui.painter().add(egui::Shape::convex_polygon(vec![c, c + vec2(5.0 * u, 0.0), c + vec2(5.0 * u, -5.0 * u)], col, Stroke::NONE));
        }
    }
    if let Some(g) = out.clicked {
        let grp = &GROUPS[g];
        let was_on = grp.tools.iter().any(|(x, _)| *x == app.ui.tool);
        if grp.tools.len() > 1 && was_on {
            st.tool_pop = Some((g, out.rects[g]));
        } else {
            let tool = last_tool(ctx, g);
            app.ui.tool = tool;
            st.tool_pop = None;
        }
    } else if let Some(g) = GROUPS.iter().position(|g| g.needs_3d).filter(|_| !has_3d)
        && ctx.input(|i| i.pointer.any_click())
        && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| out.rects[g].contains(p))
    {
        app.ui.status = "Add a 3D layer or a camera to use the camera tools".into();
    }
}

fn tool_popup(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, st: &mut MState, u: f32) {
    let Some((g, anchor)) = st.tool_pop else { return };
    let grp = &GROUPS[g];
    let rows = grp.tools.len() as f32;
    let mut pick: Option<Tool> = None;
    let cur = app.ui.tool;
    let dismissed = kit::popup(app.tokens, ctx, "m-tool-pop", anchor, full, vec2(250.0 * u, 44.0 * u * rows + 12.0 * u), u, |ui, inner| {
        for (i, (tool, label)) in grp.tools.iter().enumerate() {
            let r = Rect::from_min_size(inner.min + vec2(0.0, 44.0 * u * i as f32), vec2(inner.width(), 44.0 * u));
            let resp = kit::tap(app, ui, r, &format!("mobile.tool.{}.{}", grp.name, label), label);
            if resp.is_pointer_button_down_on() {
                ui.painter().rect_filled(r, 10.0 * u, app.tokens.pressed);
            }
            crate::icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 22.0 * u, r.center().y), vec2(22.0 * u, 22.0 * u)), tool.icon(), if *tool == cur { app.tokens.accent } else { app.tokens.icon_active });
            ui.painter().text(pos2(r.min.x + 50.0 * u, r.center().y), Align2::LEFT_CENTER, label, Tokens::medium(14.0), if *tool == cur { app.tokens.accent } else { app.tokens.text });
            if resp.clicked() {
                pick = Some(*tool);
            }
        }
    });
    if let Some(tool) = pick {
        app.ui.tool = tool;
        ctx.data_mut(|d| d.insert_temp(Id::new(("m-tool-last", g)), tool));
        st.tool_pop = None;
    } else if dismissed {
        st.tool_pop = None;
    }
}

fn menu_strip(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, st: &mut MState, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.app_bg));
    let names = crate::mobile_menu::names();
    let items: Vec<Item> = names.iter().enumerate().map(|(i, n)| Item::text(&format!("mobile.menu.{n}"), n, text_w(ui, n, u)).on(st.menu.is_some_and(|(m, _)| m == i))).collect();
    let out = kit::strip(app, ui, rect, "menus", &items, u);
    if let Some(i) = out.clicked {
        st.menu = if st.menu.is_some_and(|(m, _)| m == i) { None } else { Some((i, out.rects[i])) };
    }
}

fn menu_popup(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, st: &mut MState, u: f32) {
    let Some((i, anchor)) = st.menu else { return };
    let names = crate::mobile_menu::names();
    let Some(name) = names.get(i).cloned() else {
        st.menu = None;
        return;
    };
    let nodes = crate::mobile_menu::children_of(&name);
    let mut chosen: Option<(String, serde_json::Value)> = None;
    let w = (full.width() - 16.0 * u).min(360.0 * u);
    let h = (full.height() * 0.7).min(620.0 * u);
    let dismissed = kit::popup(app.tokens, ctx, "m-menu-pop", anchor, full, vec2(w, h), u, |ui, inner| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).id_salt(("m-menu-body", i)));
        child.set_clip_rect(inner);
        egui::ScrollArea::vertical().id_salt(("m-menu-scroll", i)).auto_shrink([false, false]).show(&mut child, |ui| {
            crate::mobile_menu::menu_tree(app, ui, inner.width(), u, &nodes, &name, 0, &mut chosen);
        });
    });
    if let Some((id, params)) = chosen {
        if !intercept(app, ctx, st, &id, &params) {
            crate::mobile_menu::run(app, ctx, Some((id, params)));
        }
        st.menu = None;
    } else if dismissed {
        st.menu = None;
    }
}

/// Phone handling of the menu commands that would open desktop-only UI (a native file dialog,
/// a docked panel): they go to the touch pages instead. Returns true when handled here.
fn intercept(app: &mut EffectcraftApp, ctx: &egui::Context, st: &mut MState, id: &str, params: &serde_json::Value) -> bool {
    let leave = |app: &mut EffectcraftApp, st: &mut MState, target: u8| {
        crate::mobile_home::goto(ctx, target);
        if app.session.is_dirty() {
            st.page = Page::ConfirmHome;
        } else {
            app.ui.start_screen = true;
        }
    };
    match id {
        "file.newProject" => leave(app, st, 1),
        "file.open" | "file.openProject" => leave(app, st, 2),
        "file.import" | "file.importMultiple" => st.page = Page::Import,
        "renderQueue.add" => st.page = Page::Export,
        "file.save" | "file.saveAs" | "file.saveCopy" | "file.incrementAndSave" => match crate::mobile_home::save_project(app, id != "file.save") {
            Ok(p) => app.ui.status = format!("Saved to {p}"),
            Err(e) => app.ui.status = format!("Not saved: {e}"),
        },
        "window.panel" => {
            let kind = params.get("panel").and_then(|v| v.as_str()).and_then(PanelKind::from_name);
            match kind {
                Some(PanelKind::Timeline) => st.plus = None,
                Some(PanelKind::Composition) => {}
                Some(k) => st.plus = Some(SECTIONS.iter().map(|(s, _)| *s).find(|s| s.panel() == k).unwrap_or(Section::Other(k))),
                None => return false,
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------------------------------
// Preview, composition navigation
// ---------------------------------------------------------------------------------------------

fn preview_area(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, rect: Rect, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.pasteboard);
    if app.session.active_comp().is_none() {
        let c = rect.center();
        ui.painter().text(pos2(c.x, c.y - 30.0 * u), Align2::CENTER_CENTER, "No composition open", Tokens::semibold(15.0), t.text);
        let b = Rect::from_center_size(pos2(c.x, c.y + 18.0 * u), vec2(200.0 * u, 44.0 * u));
        if kit::button(app, ui, b, "New composition", Btn::Primary, true, "mobile.preview.newComp", u) {
            open_page(ctx, Page::NewComp);
        }
        return;
    }
    // A tap in the preview makes it the focused panel (shortcuts, the active viewer).
    if ctx.input(|i| i.pointer.primary_pressed()) && ctx.input(|i| i.pointer.press_origin()).is_some_and(|p| rect.contains(p)) {
        app.ui.focused = PanelKind::Composition;
    }
    // Two fingers: pinch zooms (the viewer reads the zoom delta), the pair's movement pans.
    if let Some(mt) = ctx.multi_touch()
        && rect.contains(mt.start_pos)
    {
        app.ui.viewer.pan[0] += mt.translation_delta.x;
        app.ui.viewer.pan[1] += mt.translation_delta.y;
    }
    embed(app, ui, PanelKind::Composition, rect);
    // Play / pause, floating at the preview's corner.
    let d = 44.0 * u;
    let c = pos2(rect.min.x + 12.0 * u + d / 2.0, rect.max.y - 12.0 * u - d / 2.0);
    let r = Rect::from_center_size(c, vec2(d, d));
    let resp = kit::tap(app, ui, r, "mobile.play", "Play / Pause");
    ui.painter().circle_filled(c, d / 2.0, Color32::from_black_alpha(if resp.is_pointer_button_down_on() { 190 } else { 130 }));
    crate::icons::paint(ui.painter(), Rect::from_center_size(c, vec2(22.0 * u, 22.0 * u)), if app.playback.playing { Icon::Pause } else { Icon::Play }, Color32::WHITE);
    if resp.clicked() {
        let now = ctx.input(|i| i.time);
        app.toggle_play(now);
    }
}

fn comp_nav(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, rect: Rect, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, t.separator));
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.app_bg));
    let active = app.session.active_comp_id();
    let open: Vec<(u64, String)> = app.session.state.open_comps.iter().filter_map(|id| app.session.project.item(*id).map(|i| (id.0, i.name.clone()))).collect();
    let mut items: Vec<Item> = open.iter().map(|(id, name)| Item::text(&format!("mobile.nav.{id}"), name, text_w(ui, name, u)).on(active.is_some_and(|a| a.0 == *id))).collect();
    items.push(Item::icon("mobile.nav.new", Icon::Plus, 46.0 * u));
    let out = kit::strip(app, ui, rect, "nav", &items, u);
    if let Some(i) = out.clicked {
        if i < open.len() {
            let _ = app.session.execute("comp.open", json!({"comp": open[i].0}));
        } else {
            open_page(ctx, Page::NewComp);
        }
    }
}

fn splitter(ui: &mut Ui, handle: Rect, st: &mut MState, landscape: bool, body: Rect, u: f32, t: &Tokens) {
    let resp = ui.interact(handle, Id::new("m-split"), egui::Sense::drag());
    let col = if resp.dragged() { t.accent } else { t.text_faint };
    let c = handle.center();
    let (a, b) = if landscape { (vec2(0.0, -14.0 * u), vec2(0.0, 14.0 * u)) } else { (vec2(-16.0 * u, 0.0), vec2(16.0 * u, 0.0)) };
    ui.painter().line_segment([c + a, c + b], Stroke::new(4.0 * u, col));
    if resp.dragged() {
        let d = resp.drag_delta();
        let span = if landscape { body.width() } else { (body.height() - 120.0 * u).max(1.0) };
        st.split = (st.split + if landscape { d.x } else { d.y } / span.max(1.0)).clamp(0.22, 0.7);
    }
}

/// Run an existing desktop panel inside `rect`.
fn embed(app: &mut EffectcraftApp, ui: &mut Ui, kind: PanelKind, rect: Rect) {
    embed_clipped(app, ui, kind, rect, rect);
}

/// `kind` laid out in `rect`, with only `view` of it showing (and reachable by touch): `rect` may
/// be wider than the screen, as the timeline is.
fn embed_clipped(app: &mut EffectcraftApp, ui: &mut Ui, kind: PanelKind, rect: Rect, view: Rect) {
    app.auto.add(&format!("panel.{}", kind.id()), rect.intersect(view), kind.title());
    let mut child = ui.new_child(
        egui::UiBuilder::new().max_rect(rect).id_salt(("m-panel", kind.id())).ui_stack_info(egui::UiStackInfo::default().with_tag(crate::widgets::PANEL_TAG)),
    );
    child.set_clip_rect(view.intersect(ui.clip_rect()));
    panels::show(app, &mut child, kind, rect);
}

// ---------------------------------------------------------------------------------------------
// Timeline and the + panel
// ---------------------------------------------------------------------------------------------

fn timeline_header(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, rect: Rect, st: &mut MState, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.header_bg);
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], egui::Stroke::new(1.0, t.app_bg));

    let show_modes = app.ui.timeline.show_modes;
    let graph_on = app.ui.timeline.graph_editor;

    let items = vec![
        Item::text("mobile.tl.modes", if show_modes { "Modes (F4)" } else { "Switches (F4)" }, 96.0 * u).on(show_modes),
        Item::text("mobile.tl.cols", "Columns ▾", 82.0 * u).on(st.column_sheet),
        Item::text("mobile.tl.graph", "📈 Graph", 76.0 * u).on(graph_on),
        Item::text("mobile.tl.prev_key", "◀ Key (J)", 76.0 * u),
        Item::text("mobile.tl.key", "◆ Key", 58.0 * u),
        Item::text("mobile.tl.next_key", "Key (K) ▶", 76.0 * u),
        Item::text("mobile.tl.twirl", "▾ Twirl", 64.0 * u),
    ];
    let out = kit::strip(app, ui, rect, "tl_header", &items, u);
    if let Some(i) = out.clicked {
        match items[i].id.as_str() {
            "mobile.tl.modes" => {
                app.ui.timeline.show_modes = !app.ui.timeline.show_modes;
            }
            "mobile.tl.cols" => {
                st.column_sheet = !st.column_sheet;
            }
            "mobile.tl.graph" => {
                app.ui.timeline.graph_editor = !app.ui.timeline.graph_editor;
            }
            "mobile.tl.prev_key" => {
                let _ = menus::invoke(app, ctx, "time.previousKey", json!({}));
            }
            "mobile.tl.key" => {
                let _ = menus::invoke(app, ctx, "prop.toggleKey", json!({}));
            }
            "mobile.tl.next_key" => {
                let _ = menus::invoke(app, ctx, "time.nextKey", json!({}));
            }
            "mobile.tl.twirl" => {
                let _ = menus::invoke(app, ctx, "timeline.twirlSelected", json!({}));
            }
            _ => {}
        }
    }
}

fn column_sheet(app: &mut EffectcraftApp, ctx: &egui::Context, bounds: Rect, u: f32) -> bool {
    let t = app.tokens;
    let mut close = false;
    let h = (bounds.height() * 0.75).min(560.0 * u);
    let dismissed = kit::sheet(t, ctx, "m-columns-sheet", bounds, h, u, |ui, inner| {
        let w = inner.width();
        let head = Rect::from_min_size(inner.min, vec2(w, 44.0 * u));
        let p = ui.painter();
        p.text(pos2(head.min.x + 8.0 * u, head.center().y), Align2::LEFT_CENTER, "Timeline Columns", Tokens::semibold(16.0), t.text);
        let x = Rect::from_min_size(pos2(head.max.x - 44.0 * u, head.min.y), vec2(44.0 * u, head.height()));
        if kit::icon_tap(app, ui, x, Icon::Close, "Close", false, true, "mobile.cols.close", u) {
            close = true;
        }
        let body = Rect::from_min_max(pos2(inner.min.x, head.max.y + 4.0 * u), inner.max);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body).id_salt("m-cols-body"));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical().id_salt("m-cols-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            kit::heading(app, ui, w, "Show / Hide Columns", u);

            const ALL_COLS: [(&str, &str, &str); 10] = [
                ("parent", "Parent & Link", "Link layers to each other"),
                ("switches", "Layer Switches", "Shy, Lock, 3D, Motion Blur..."),
                ("modes", "Blend Modes & Matte", "38 Blending modes and Track Matte"),
                ("keys", "Keyframe Navigator", "Prev/Next keyframe buttons"),
                ("in", "In Point", "Start timestamp of layer"),
                ("out", "Out Point", "End timestamp of layer"),
                ("duration", "Duration", "Layer duration"),
                ("stretch", "Time Stretch", "Speed/stretch percentage"),
                ("num", "Layer Number", "Sequential layer numbering"),
                ("comment", "Comments", "User layer notes"),
            ];

            for (col_id, title, desc) in ALL_COLS {
                let is_on = panels::timeline::column_visible(&app.ui.timeline, col_id);
                if kit::list_row(app, ui, w, 48.0 * u, None, title, Some(desc), Some(is_on), true, &format!("mobile.col.toggle.{col_id}"), u) {
                    let _ = panels::timeline::set_column(&mut app.ui.timeline, col_id, !is_on);
                }
            }

            ui.add_space(12.0 * u);
            let btn_r = Rect::from_min_size(pos2(inner.min.x + 8.0 * u, ui.cursor().min.y), vec2(w - 16.0 * u, 42.0 * u));
            if kit::button(app, ui, btn_r, "Reset to Mobile Defaults (Compact)", Btn::Secondary, true, "mobile.cols.reset", u) {
                for id in ["keys", "num", "comment", "switches", "modes", "parent", "in", "out", "duration", "stretch"] {
                    let _ = panels::timeline::set_column(&mut app.ui.timeline, id, false);
                }
                app.ui.timeline.show_modes = false;
                close = true;
            }
            ui.add_space(16.0 * u);
        });
    });
    close || dismissed
}

fn timeline_area(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, st: &mut MState, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    if app.session.active_comp().is_none() {
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Create a composition to start", Tokens::ui(13.0), t.text_dim);
        return;
    }
    app.ui.focused = PanelKind::Timeline;
    let ctx = ui.ctx().clone();

    // The top strip provides quick mobile controls: Switches/Modes toggle, Columns picker, Graph, Key navigation
    let header_h = 36.0 * u;
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), header_h));
    timeline_header(app, ui, &ctx, header, st, u);

    // The whole timeline (layer names, properties and the time graph) is wider than the screen and
    // slides sideways: by the bar under it, or two fingers over the names (or over a graph that
    // shows the whole comp; zoomed in, two fingers pan the time instead).
    let bar_h = 36.0 * u; // a finger-sized strip; only its slim track is drawn
    let view = Rect::from_min_max(pos2(rect.min.x, header.max.y), pos2(rect.max.x, rect.max.y - bar_h));
    let bar = Rect::from_min_max(pos2(rect.min.x, view.max.y), rect.max);
    let extra = (view.width() * 0.9).round();
    let finger: f32 = ctx.data_mut(|d| d.remove_temp::<f32>(Id::new("mobile-tl-pan-dx"))).unwrap_or(0.0);
    st.tl_pan = (st.tl_pan + finger).clamp(0.0, extra);
    let wide = Rect::from_min_size(pos2(view.min.x - st.tl_pan, view.min.y), vec2(view.width() + extra, view.height()));
    ctx.data_mut(|d| d.insert_temp(Id::new("mobile-tl-view"), view));
    embed_clipped(app, ui, PanelKind::Timeline, wide, view);
    pan_bar(app, ui, bar, view.width(), extra, st, u);

    // The floating +: swaps the timeline for the section panel. Drag it where it is out of the way.
    let d = 54.0 * u;
    let reach = view.shrink2(vec2(d / 2.0, d / 2.0));
    let corner = pos2(view.max.x - 16.0 * u - d / 2.0, view.max.y - 30.0 * u - d / 2.0);
    let mut c = match st.plus_pos {
        Some(f) => pos2(view.min.x + f.x * view.width(), view.min.y + f.y * view.height()),
        None => corner,
    };
    c = pos2(c.x.clamp(reach.min.x, reach.max.x.max(reach.min.x)), c.y.clamp(reach.min.y, reach.max.y.max(reach.min.y)));
    let r = Rect::from_center_size(c, vec2(d, d));
    app.auto.add("mobile.plus", r, "Panels");
    let resp = ui.interact(r, Id::new(("m", "mobile.plus")), egui::Sense::click_and_drag());
    if resp.dragged() {
        c += resp.drag_delta();
        c = pos2(c.x.clamp(reach.min.x, reach.max.x.max(reach.min.x)), c.y.clamp(reach.min.y, reach.max.y.max(reach.min.y)));
        st.plus_pos = Some(vec2((c.x - view.min.x) / view.width().max(1.0), (c.y - view.min.y) / view.height().max(1.0)));
    }
    ui.painter().circle_filled(c + vec2(0.0, 2.0 * u), d / 2.0, Color32::from_black_alpha(100));
    ui.painter().circle_filled(c, d / 2.0, if resp.is_pointer_button_down_on() { t.accent_hover } else { t.accent });
    crate::icons::paint(ui.painter(), Rect::from_center_size(c, vec2(26.0 * u, 26.0 * u)), Icon::Plus, Color32::WHITE);
    if resp.clicked() {
        st.plus = Some(Section::Assets);
    }
}

/// The slider under the timeline: its thumb is the part of the (wider) timeline that shows.
fn pan_bar(app: &mut EffectcraftApp, ui: &mut Ui, bar: Rect, shown: f32, extra: f32, st: &mut MState, u: f32) {
    let t = app.tokens;
    app.auto.add("mobile.timeline.pan", bar, "Slide timeline");
    let resp = ui.interact(bar, Id::new(("m", "timeline-pan")), egui::Sense::drag());
    let track = bar.shrink2(vec2(10.0 * u, (bar.height() - 10.0 * u) / 2.0));
    let thumb_w = (track.width() * shown / (shown + extra)).max(36.0 * u).min(track.width());
    let room = (track.width() - thumb_w).max(1.0);
    if resp.dragged() {
        st.tl_pan = (st.tl_pan + resp.drag_delta().x * extra / room).clamp(0.0, extra);
    }
    let x = track.min.x + room * (st.tl_pan / extra.max(1.0));
    let thumb = Rect::from_min_size(pos2(x, track.min.y), vec2(thumb_w, track.height()));
    ui.painter().rect_filled(bar, 0.0, t.app_bg);
    ui.painter().rect_filled(track, track.height() / 2.0, t.pressed);
    ui.painter().rect_filled(thumb, thumb.height() / 2.0, if resp.dragged() { t.accent_hover } else { t.accent });
}

fn plus_panel(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, st: &mut MState, sec: Section, u: f32) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    let head = Rect::from_min_size(rect.min, vec2(rect.width(), 46.0 * u));
    ui.painter().rect_filled(head, 0.0, t.header_bg);
    ui.painter().line_segment([head.left_bottom(), head.right_bottom()], Stroke::new(1.0, t.app_bg));
    // Back to the timeline, then the scrolling sections.
    let back = Rect::from_min_size(head.min, vec2(48.0 * u, head.height()));
    if kit::icon_tap(app, ui, back, Icon::Close, "Back to timeline", false, true, "mobile.plus.close", u) {
        st.plus = None;
        return;
    }
    let strip = Rect::from_min_max(pos2(back.max.x, head.min.y), head.max);
    // A panel opened from the Window menu that is not one of the usual sections gets its own tab.
    let mut secs: Vec<(Section, String)> = SECTIONS.iter().map(|(s, l)| (*s, l.to_string())).collect();
    if !secs.iter().any(|(s, _)| *s == sec) {
        secs.insert(0, (sec, sec.panel().title().to_string()));
    }
    let items: Vec<Item> = secs.iter().map(|(s, label)| Item::text(&format!("mobile.plus.{label}"), label, text_w(ui, label, u)).on(*s == sec)).collect();
    let out = kit::strip(app, ui, strip, "plus", &items, u);
    if let Some(i) = out.clicked {
        st.plus = Some(secs[i].0);
    }
    let body = Rect::from_min_max(pos2(rect.min.x, head.max.y), rect.max);
    app.ui.focused = sec.panel();
    embed(app, ui, sec.panel(), body);
}
