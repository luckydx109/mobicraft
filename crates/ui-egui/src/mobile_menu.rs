//! Menus for fingers: the application menus (File … Help) as lists with expandable submenus,
//! and the layer action sheet that replaces the desktop's right-click menu. Both are built from
//! the same menu definitions as the desktop menu bar (`effectcraft_engine::menus`), so every
//! entry keeps its label, enabled state and check mark, and runs the same command.

use effectcraft_engine::menus::MenuNode;
use egui::{Align2, Color32, Id, Rect, Sense, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::icons::Icon;
use crate::mobile_kit::{self as kit, Btn};
use crate::theme::Tokens;
use crate::{EffectcraftApp, menus};

type Chosen = Option<(String, Value)>;

/// Entries the phone app does not offer: the motion-tracking features that were removed on
/// purpose, and desktop-only actions that need a native file manager, scripting or a dock layout.
pub fn hidden(e: &effectcraft_engine::menus::MenuEntry) -> bool {
    const IDS: [&str; 10] = [
        "file.installScript",
        "file.installScriptUIPanel",
        "file.runScript",
        "file.revealInFinder",
        "file.watchFolder",
        "app.quit",
        "file.importVanishingPoint",
        "window.resetWorkspace",
        "window.saveWorkspace",
        "window.deleteWorkspace",
    ];
    const EFFECTS: [&str; 4] = ["Warp Stabilizer", "3D Camera Tracker", "Face Track Points", "Roto Brush & Refine Edge"];
    e.command.starts_with("track.")
        || IDS.contains(&e.command.as_str())
        || (e.command == "effect.apply" && EFFECTS.contains(&e.label.as_str()))
        || (e.command == "window.panel" && e.label == "Tracker")
        || (e.command == "window.workspace")
}

/// Draw `nodes` as touch rows; a chosen entry goes to `chosen`. Submenus expand in place.
pub fn menu_tree(app: &mut EffectcraftApp, ui: &mut Ui, width: f32, u: f32, nodes: &[MenuNode], key: &str, depth: usize, chosen: &mut Chosen) {
    let t = app.tokens;
    let indent = 14.0 * u * depth as f32;
    for (i, n) in nodes.iter().enumerate() {
        match n {
            MenuNode::Separator => {
                let (r, _) = ui.allocate_exact_size(vec2(width, 9.0 * u), Sense::hover());
                ui.painter().line_segment([pos2(r.min.x + 12.0 * u + indent, r.center().y), pos2(r.max.x - 12.0 * u, r.center().y)], egui::Stroke::new(1.0, t.separator));
            }
            MenuNode::Item(e) if hidden(e) => {}
            MenuNode::Item(e) => {
                let (label, en, check) = (menus::entry_label(app, e), menus::entry_enabled(app, e), menus::entry_checked(app, e));
                let id = format!("mobile.menu.{key}.{}", e.command);
                let mut hit = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(indent);
                    hit = kit::list_row(app, ui, width - indent, 46.0 * u, None, &label, None, check, en, &format!("{id}.{i}"), u);
                });
                if hit {
                    *chosen = Some((e.command.clone(), e.params_or_empty()));
                }
            }
            MenuNode::Dynamic { name } => {
                let ws = app.ui.workspace.clone();
                let saved = app.saved_workspace_names();
                let (entries, empty) = effectcraft_engine::menus::dynamic(&app.session, name, &menus::dyn_ctx(&ws, &saved));
                if entries.is_empty()
                    && let Some(msg) = empty
                {
                    kit::list_row(app, ui, width, 40.0 * u, None, &msg, None, None, false, &format!("mobile.menu.{key}.{name}.empty"), u);
                }
                for (j, e) in entries.iter().enumerate().filter(|(_, e)| !hidden(e)) {
                    let id = format!("mobile.menu.{key}.{name}.{j}");
                    if kit::list_row(app, ui, width - indent, 46.0 * u, None, &e.label, None, menus::entry_checked(app, e), menus::entry_enabled(app, e), &id, u) {
                        *chosen = Some((e.command.clone(), e.params_or_empty()));
                    }
                }
            }
            MenuNode::Submenu { label, .. } if depth <= 1 && (label == "Workspace" || label == "Motion Tracking") => {}
            MenuNode::Submenu { label, children } => {
                let ws = app.ui.workspace.clone();
                let shown = crate::i18n::submenu(app, label, effectcraft_engine::menus::submenu_label(&app.session, label, &menus::dyn_ctx(&ws, &[])));
                let sid = Id::new(("m-menu-open", key, label.as_str(), depth));
                let open: bool = ui.data(|d| d.get_temp(sid)).unwrap_or(false);
                let mut hit = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(indent);
                    hit = kit::list_row(app, ui, width - indent, 46.0 * u, None, &shown, None, None, true, &format!("mobile.menu.{key}.sub.{label}"), u);
                    let r = ui.min_rect();
                    let c = pos2(r.max.x - 22.0 * u, r.max.y - 23.0 * u);
                    crate::icons::paint(ui.painter(), Rect::from_center_size(c, vec2(18.0 * u, 18.0 * u)), if open { Icon::ChevronDown } else { Icon::ChevronRight }, t.text_faint);
                });
                if hit {
                    ui.data_mut(|d| d.insert_temp(sid, !open));
                }
                if open {
                    menu_tree(app, ui, width, u, children, &format!("{key}.{label}"), depth + 1, chosen);
                }
            }
        }
    }
}

/// The top-level menu names, in order.
pub fn names() -> Vec<String> {
    effectcraft_engine::menus::menu_bar()
        .iter()
        .filter_map(|n| match n {
            MenuNode::Submenu { label, .. } => Some(label.clone()),
            _ => None,
        })
        .collect()
}

/// The children of top-level menu `name`.
pub fn children_of(name: &str) -> Vec<MenuNode> {
    effectcraft_engine::menus::menu_bar()
        .iter()
        .find_map(|n| match n {
            MenuNode::Submenu { label, children } if label == name => Some(children.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// A submenu of the Layer menu (`Arrange`, `Blending Mode`, …).
fn layer_sub(name: &str) -> Vec<MenuNode> {
    children_of("Layer")
        .into_iter()
        .find_map(|n| match n {
            MenuNode::Submenu { label, children } if label == name => Some(children),
            _ => None,
        })
        .unwrap_or_default()
}

/// Run a menu choice through the same path as the desktop menu bar.
pub fn run(app: &mut EffectcraftApp, ctx: &egui::Context, chosen: Chosen) {
    if let Some((id, params)) = chosen
        && let Err(e) = menus::invoke(app, ctx, &id, if params.is_null() { json!({}) } else { params })
    {
        app.ui.status = e;
    }
}

const CREATE: [(&str, Icon, &str); 7] = [
    ("layer.newText", Icon::TextLayer, "Text"),
    ("layer.newSolid", Icon::Solid, "Solid"),
    ("layer.newLight", Icon::Light, "Light"),
    ("layer.newCamera", Icon::Camera, "Camera"),
    ("layer.newNull", Icon::Null, "Null Object"),
    ("layer.newShape", Icon::ShapeLayer, "Shape Layer"),
    ("layer.newAdjustment", Icon::Adjustment, "Adjustment Layer"),
];

/// Layer switches worth a touch toggle: key, label.
const SWITCHES: [(&str, &str); 8] = [
    ("threeD", "3D Layer"),
    ("motionBlur", "Motion Blur"),
    ("adjustment", "Adjustment"),
    ("shy", "Shy"),
    ("fx", "Effects"),
    ("quality", "Best Quality"),
    ("frameBlend", "Frame Blend"),
    ("collapse", "Collapse"),
];

/// The 16 label colors from After Effects (index 0 = None).
const LABEL_COLORS: [(u8, &str, Color32); 17] = [
    (0, "None", Color32::from_rgb(0x80, 0x80, 0x80)),
    (1, "Red", Color32::from_rgb(0xE0, 0x40, 0x40)),
    (2, "Yellow", Color32::from_rgb(0xE8, 0xD4, 0x40)),
    (3, "Aqua", Color32::from_rgb(0x60, 0xD0, 0xD0)),
    (4, "Pink", Color32::from_rgb(0xE0, 0x80, 0xC0)),
    (5, "Lavender", Color32::from_rgb(0xB0, 0x90, 0xD0)),
    (6, "Peach", Color32::from_rgb(0xE0, 0xA0, 0x70)),
    (7, "Sea Foam", Color32::from_rgb(0x70, 0xD0, 0x90)),
    (8, "Blue", Color32::from_rgb(0x50, 0x90, 0xE0)),
    (9, "Green", Color32::from_rgb(0x50, 0xB0, 0x50)),
    (10, "Purple", Color32::from_rgb(0x90, 0x60, 0xC0)),
    (11, "Orange", Color32::from_rgb(0xE0, 0x80, 0x30)),
    (12, "Brown", Color32::from_rgb(0xA0, 0x70, 0x40)),
    (13, "Fuchsia", Color32::from_rgb(0xD0, 0x40, 0x90)),
    (14, "Cyan", Color32::from_rgb(0x40, 0xD0, 0xE0)),
    (15, "Sandstone", Color32::from_rgb(0xC0, 0xB0, 0x80)),
    (16, "Dark Green", Color32::from_rgb(0x30, 0x80, 0x40)),
];

/// The groups that open a submenu inside the sheet.
const GROUPS: [(&str, &str); 8] = [
    ("Arrange", "Arrange"),
    ("Blending Mode", "Blending Mode"),
    ("Track Matte", "Track Matte"),
    ("Layer Styles", "Layer Styles"),
    ("Time", "Time"),
    ("Transform", "Transform"),
    ("Mask", "Mask"),
    ("Create", "Create"),
];

/// The layer action sheet for `layer`: quick toggles (visible/solo/lock), create, operations,
/// parent picker, label color, settings and switches. Returns true when it should close.
pub fn layer_sheet(app: &mut EffectcraftApp, ctx: &egui::Context, bounds: Rect, layer: u64, u: f32) -> bool {
    let t = app.tokens;
    let Some((name, icon, is_comp, switches, vis, solo, locked, parent_id, parent_name, label_idx, all_layers, blend_label, matte_label)) = app.session.active_comp().and_then(|c| {
        c.layer(effectcraft_engine::project::LayerId(layer)).map(|l| {
            use effectcraft_engine::project::LayerSource as S;
            let icon = match &l.source {
                S::Footage { .. } => Icon::Footage,
                S::Comp { .. } => Icon::Comp,
                S::Solid { .. } if l.switches.adjustment => Icon::Adjustment,
                S::Solid { .. } => Icon::Solid,
                S::Text => Icon::TextLayer,
                S::Shape => Icon::ShapeLayer,
                S::Camera => Icon::Camera,
                S::Light { .. } => Icon::Light,
                S::Model { .. } | S::Primitive { .. } => Icon::Cube,
                S::Null => Icon::Null,
            };
            let sw = &l.switches;
            let on = [l.is_3d(), sw.motion_blur, sw.adjustment, sw.shy, sw.effects, sw.quality == effectcraft_engine::project::Quality::Best, sw.frame_blend != effectcraft_engine::project::FrameBlend::Off, sw.collapse];
            let parent_id = l.parent;
            let parent_name = l.parent.and_then(|pid| c.layer(pid).map(|pl| pl.name.clone()));
            let label_idx = l.label as u8;
            let blend_label = l.blend_mode.label().to_string();
            let matte_label = l.track_matte.map(|tm| tm.kind.label()).unwrap_or("None").to_string();
            // Collect all other layers in the comp for the parent picker.
            let all_layers: Vec<(u64, String)> = c.layers.iter()
                .filter(|ol| ol.id.0 != layer)
                .map(|ol| (ol.id.0, ol.name.clone()))
                .collect();
            (l.name.clone(), icon, matches!(l.source, S::Comp { .. }), on, sw.video, sw.solo, sw.locked, parent_id, parent_name, label_idx, all_layers, blend_label, matte_label)
        })
    }) else {
        return true;
    };

    let sub_id = Id::new("m-layer-sub");
    let sub: Option<String> = ctx.data(|d| d.get_temp(sub_id));
    let parent_sub_id = Id::new("m-layer-parent-open");
    let parent_open: bool = ctx.data(|d| d.get_temp(parent_sub_id)).unwrap_or(false);
    let label_sub_id = Id::new("m-layer-label-open");
    let label_open: bool = ctx.data(|d| d.get_temp(label_sub_id)).unwrap_or(false);
    let blend_sub_id = Id::new("m-layer-blend-open");
    let blend_open: bool = ctx.data(|d| d.get_temp(blend_sub_id)).unwrap_or(false);
    let matte_sub_id = Id::new("m-layer-matte-open");
    let matte_open: bool = ctx.data(|d| d.get_temp(matte_sub_id)).unwrap_or(false);
    let styles_sub_id = Id::new("m-layer-styles-open");
    let styles_open: bool = ctx.data(|d| d.get_temp(styles_sub_id)).unwrap_or(false);
    let timing_sub_id = Id::new("m-layer-timing-open");
    let timing_open: bool = ctx.data(|d| d.get_temp(timing_sub_id)).unwrap_or(false);
    let keys_sub_id = Id::new("m-layer-keys-open");
    let keys_open: bool = ctx.data(|d| d.get_temp(keys_sub_id)).unwrap_or(false);

    let mut close = false;
    let mut chosen: Chosen = None;
    let mut cmd: Option<(&'static str, Value)> = None;
    let mut back = false;
    let mut open_comp: Option<u64> = None;
    let comp_item = app.session.active_comp().and_then(|c| match c.layer(effectcraft_engine::project::LayerId(layer)).map(|l| &l.source) {
        Some(effectcraft_engine::project::LayerSource::Comp { item }) if is_comp => Some(item.0),
        _ => None,
    });

    let h = (bounds.height() * 0.85).min(720.0 * u);
    let dismissed = kit::sheet(app.tokens, ctx, "m-layer-sheet", bounds, h, u, |ui, inner| {
        let w = inner.width();
        let head = Rect::from_min_size(inner.min, vec2(w, 44.0 * u));
        let p = ui.painter().clone();
        let (title, show_back) = match &sub {
            Some(s) => (s.clone(), true),
            None => (name.clone(), false),
        };
        if show_back {
            let b = Rect::from_min_size(head.min, vec2(44.0 * u, head.height()));
            if kit::icon_tap(app, ui, b, Icon::ChevronLeft, "Back", false, true, "mobile.layer.back", u) {
                back = true;
            }
            p.text(pos2(b.max.x + 4.0 * u, head.center().y), Align2::LEFT_CENTER, title, Tokens::semibold(16.0), t.text);
        } else {
            crate::icons::paint(&p, Rect::from_center_size(pos2(head.min.x + 18.0 * u, head.center().y), vec2(22.0 * u, 22.0 * u)), icon, t.icon_active);
            crate::widgets::text_fit(&p, pos2(head.min.x + 40.0 * u, head.center().y), Align2::LEFT_CENTER, &title, Tokens::semibold(16.0), w - 100.0 * u, t.text);
        }
        let x = Rect::from_min_size(pos2(head.max.x - 44.0 * u, head.min.y), vec2(44.0 * u, head.height()));
        if kit::icon_tap(app, ui, x, Icon::Close, "Close", false, true, "mobile.layer.close", u) {
            close = true;
        }
        let body = Rect::from_min_max(pos2(inner.min.x, head.max.y + 4.0 * u), inner.max);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body).id_salt("m-layer-body"));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical().id_salt("m-layer-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            if let Some(s) = &sub {
                let nodes = layer_sub(s);
                if nodes.is_empty() {
                    kit::list_row(app, ui, w, 44.0 * u, None, "Nothing available for this layer", None, None, false, "mobile.layer.empty", u);
                }
                menu_tree(app, ui, w, u, &nodes, &format!("layer.{s}"), 0, &mut chosen);
                return;
            }

            // ─── Quick Toggles (Visible / Solo / Lock) ─────────────────────
            let gap = 8.0 * u;
            {
                let cols = 3usize;
                // The scroll area clips a few points short of the sheet's width: keep the third pill inside.
                let wq = w - 12.0 * u;
                let cw = (wq - gap * (cols as f32 - 1.0)) / cols as f32;
                let (row, _) = ui.allocate_exact_size(vec2(w, 46.0 * u + gap), Sense::hover());
                let toggles: [(&str, &str, Icon, bool); 3] = [
                    ("eye", "Visible", Icon::Eye, vis),
                    ("solo", "Solo", Icon::Solo, solo),
                    ("lock", "Lock", Icon::Lock, locked),
                ];
                for (i, (key, label, ic, on)) in toggles.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(row.min.x + (cw + gap) * i as f32, row.min.y), vec2(cw, 46.0 * u));
                    let resp = kit::tap(app, ui, r, &format!("mobile.layer.quick.{key}"), label);
                    let fill = if *on { t.accent } else if resp.is_pointer_button_down_on() { t.pressed } else { t.hover };
                    ui.painter().rect_filled(r, r.height() / 2.0, fill);
                    let icon_r = Rect::from_center_size(pos2(r.center().x - 28.0 * u, r.center().y), vec2(20.0 * u, 20.0 * u));
                    crate::icons::paint(ui.painter(), icon_r, *ic, if *on { Color32::WHITE } else { t.text });
                    ui.painter().text(pos2(r.center().x + 4.0 * u, r.center().y), Align2::LEFT_CENTER, label, Tokens::medium(13.0), if *on { Color32::WHITE } else { t.text });
                    if resp.clicked() {
                        cmd = Some(("layer.setSwitch", json!({"layers": [layer], "switch": key})));
                    }
                }
            }

            // ─── Blending Mode ─────────────────────────────────────────────
            kit::heading(app, ui, w, "Blending Mode", u);
            {
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Channel), &format!("Mode: {blend_label}"), Some(if blend_open { "▾" } else { "›" }), None, true, "mobile.layer.blend.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(blend_sub_id, !blend_open));
                }
                if blend_open {
                    const BLEND_CATEGORIES: [(&str, &[&str]); 6] = [
                        ("Standard", &["Normal", "Dissolve", "Dancing Dissolve"]),
                        ("Darken", &["Darken", "Multiply", "Color Burn", "Classic Color Burn", "Linear Burn", "Darker Color"]),
                        ("Lighten", &["Add", "Lighten", "Screen", "Color Dodge", "Classic Color Dodge", "Linear Dodge", "Lighter Color"]),
                        ("Contrast", &["Overlay", "Soft Light", "Hard Light", "Linear Light", "Vivid Light", "Pin Light", "Hard Mix"]),
                        ("Inversion", &["Difference", "Classic Difference", "Exclusion", "Subtract", "Divide"]),
                        ("Color & Matte", &["Hue", "Saturation", "Color", "Luminosity", "Stencil Alpha", "Stencil Luma", "Silhouette Alpha", "Silhouette Luma", "Alpha Add", "Luminescent Premul"]),
                    ];
                    let bcols = 2usize;
                    let bcw = (w - gap) / bcols as f32;
                    for (cat_name, modes) in BLEND_CATEGORIES {
                        ui.add_space(4.0 * u);
                        let (hdr_rect, _) = ui.allocate_exact_size(vec2(w, 20.0 * u), Sense::hover());
                        ui.painter().text(pos2(hdr_rect.min.x + 6.0 * u, hdr_rect.center().y), Align2::LEFT_CENTER, cat_name, Tokens::semibold(11.5), t.text_faint);
                        for chunk in modes.chunks(bcols) {
                            let (row, _) = ui.allocate_exact_size(vec2(w, 40.0 * u + gap), Sense::hover());
                            for (i, mode_name) in chunk.iter().enumerate() {
                                let r = Rect::from_min_size(pos2(row.min.x + (bcw + gap) * i as f32, row.min.y), vec2(bcw, 40.0 * u));
                                let is_cur = *mode_name == blend_label;
                                let resp = kit::tap(app, ui, r, &format!("mobile.layer.blend.{mode_name}"), mode_name);
                                let fill = if is_cur { t.accent } else if resp.is_pointer_button_down_on() { t.pressed } else { t.hover };
                                ui.painter().rect_filled(r, 8.0 * u, fill);
                                crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, mode_name, Tokens::medium(12.0), bcw - 8.0 * u, if is_cur { Color32::WHITE } else { t.text });
                                if resp.clicked() {
                                    cmd = Some(("layer.setBlendMode", json!({"layers": [layer], "mode": mode_name})));
                                    ctx.data_mut(|d| d.insert_temp(blend_sub_id, false));
                                }
                            }
                        }
                    }
                }
            }

            // ─── Track Matte ───────────────────────────────────────────────
            kit::heading(app, ui, w, "Track Matte", u);
            {
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::MaskVis), &format!("Matte: {matte_label}"), Some(if matte_open { "▾" } else { "›" }), None, true, "mobile.layer.matte.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(matte_sub_id, !matte_open));
                }
                if matte_open {
                    let mattes: [(&str, &str); 7] = [
                        ("none", "No Track Matte"),
                        ("alpha", "Alpha Matte"),
                        ("alphaInverted", "Alpha Inverted Matte"),
                        ("luma", "Luma Matte"),
                        ("lumaInverted", "Luma Inverted Matte"),
                        ("above", "Matte with Layer Above"),
                        ("below", "Matte with Layer Below"),
                    ];
                    for (op, mlbl) in mattes {
                        let is_cur = mlbl == matte_label || (op == "none" && matte_label == "None");
                        if kit::list_row(app, ui, w, 44.0 * u, None, mlbl, None, Some(is_cur), true, &format!("mobile.layer.matte.{op}"), u) {
                            cmd = Some(("layer.trackMatte", json!({"layers": [layer], "op": op})));
                            ctx.data_mut(|d| d.insert_temp(matte_sub_id, false));
                        }
                    }
                }
            }

            // ─── Parent & Link ─────────────────────────────────────────────
            kit::heading(app, ui, w, "Parent & Link", u);
            {
                let parent_label = match &parent_name {
                    Some(n) => format!("Parent: {n}"),
                    None => "No Parent (set one)".to_string(),
                };
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Link), &parent_label, Some(if parent_open { "▾" } else { "›" }), None, true, "mobile.layer.parent.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(parent_sub_id, !parent_open));
                }
                if parent_open {
                    // "None" option to remove parent
                    if parent_id.is_some()
                        && kit::list_row(app, ui, w, 44.0 * u, Some(Icon::Close), "Remove Parent", None, None, true, "mobile.layer.parent.none", u)
                    {
                        cmd = Some(("layer.setParent", json!({"layers": [layer], "parent": null})));
                        ctx.data_mut(|d| d.insert_temp(parent_sub_id, false));
                    }
                    for (lid, lname) in &all_layers {
                        let is_cur = parent_id.is_some_and(|p| p.0 == *lid);
                        if kit::list_row(app, ui, w, 44.0 * u, Some(Icon::Null), lname, None, Some(is_cur), true, &format!("mobile.layer.parent.{lid}"), u) {
                            cmd = Some(("layer.setParent", json!({"layers": [layer], "parent": lid})));
                            ctx.data_mut(|d| d.insert_temp(parent_sub_id, false));
                        }
                    }
                }
            }

            // ─── Layer Styles ──────────────────────────────────────────────
            kit::heading(app, ui, w, "Layer Styles", u);
            {
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Fx), "Layer Styles (Drop Shadow, Glow...)", Some(if styles_open { "▾" } else { "›" }), None, true, "mobile.layer.styles.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(styles_sub_id, !styles_open));
                }
                if styles_open {
                    let style_items: [(&'static str, &'static str); 9] = [
                        ("layer.style.dropShadow", "Drop Shadow"),
                        ("layer.style.innerShadow", "Inner Shadow"),
                        ("layer.style.outerGlow", "Outer Glow"),
                        ("layer.style.innerGlow", "Inner Glow"),
                        ("layer.style.bevelEmboss", "Bevel & Emboss"),
                        ("layer.style.satin", "Satin"),
                        ("layer.style.colorOverlay", "Color Overlay"),
                        ("layer.style.gradientOverlay", "Gradient Overlay"),
                        ("layer.style.stroke", "Stroke"),
                    ];
                    let scols = 2usize;
                    let scw = (w - gap) / scols as f32;
                    for chunk in style_items.chunks(scols) {
                        let (row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                        for (i, (scmd, slbl)) in chunk.iter().enumerate() {
                            let r = Rect::from_min_size(pos2(row.min.x + (scw + gap) * i as f32, row.min.y), vec2(scw, 42.0 * u));
                            let resp = kit::tap(app, ui, r, &format!("mobile.layer.style.{slbl}"), slbl);
                            ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
                            crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, slbl, Tokens::medium(12.0), scw - 8.0 * u, t.text);
                            if resp.clicked() {
                                cmd = Some((scmd, json!({"layers": [layer]})));
                            }
                        }
                    }
                    let (ctrl_row, _) = ui.allocate_exact_size(vec2(w, 40.0 * u + gap), Sense::hover());
                    let cw3 = (w - gap * 2.0) / 3.0;
                    let ctrls: [(&'static str, &'static str); 3] = [
                        ("layer.style.showAll", "Show All"),
                        ("layer.style.removeAll", "Remove All"),
                        ("layer.style.convertToEditable", "Editable"),
                    ];
                    for (i, (ccmd, clbl)) in ctrls.iter().enumerate() {
                        let r = Rect::from_min_size(pos2(ctrl_row.min.x + (cw3 + gap) * i as f32, ctrl_row.min.y), vec2(cw3, 40.0 * u));
                        let resp = kit::tap(app, ui, r, &format!("mobile.layer.style.ctrl.{clbl}"), clbl);
                        ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.field_bg });
                        crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, clbl, Tokens::medium(11.0), cw3 - 6.0 * u, t.text);
                        if resp.clicked() {
                            cmd = Some((ccmd, json!({"layers": [layer]})));
                        }
                    }
                }
            }

            // ─── Keyframe Animation & Easing ───────────────────────────────
            kit::heading(app, ui, w, "Keyframe Animation & Easing", u);
            {
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Keyframe), "Easing & Keyframe Jump", Some(if keys_open { "▾" } else { "›" }), None, true, "mobile.layer.keys.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(keys_sub_id, !keys_open));
                }
                if keys_open {
                    let kcw = (w - gap * 2.0) / 3.0;
                    // Row 1: Keyframe navigation
                    let (nav_row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                    let navs: [(&'static str, &'static str); 3] = [
                        ("time.previousKey", "◀ Prev Key (J)"),
                        ("prop.toggleKey", "◆ Toggle Key"),
                        ("time.nextKey", "Next Key (K) ▶"),
                    ];
                    for (i, (kcmd, klbl)) in navs.iter().enumerate() {
                        let r = Rect::from_min_size(pos2(nav_row.min.x + (kcw + gap) * i as f32, nav_row.min.y), vec2(kcw, 42.0 * u));
                        let resp = kit::tap(app, ui, r, &format!("mobile.layer.keys.{kcmd}"), klbl);
                        ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.field_bg });
                        crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, klbl, Tokens::medium(11.0), kcw - 4.0 * u, t.text);
                        if resp.clicked() {
                            cmd = Some((kcmd, json!({})));
                        }
                    }
                    // Row 2: Easing
                    let (ease_row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                    let eases: [(&'static str, &'static str); 3] = [
                        ("keys.easyEase", "Easy Ease (F9)"),
                        ("keys.easyEaseIn", "Ease In (⇧F9)"),
                        ("keys.easyEaseOut", "Ease Out (⌘⇧F9)"),
                    ];
                    for (i, (kcmd, klbl)) in eases.iter().enumerate() {
                        let r = Rect::from_min_size(pos2(ease_row.min.x + (kcw + gap) * i as f32, ease_row.min.y), vec2(kcw, 42.0 * u));
                        let resp = kit::tap(app, ui, r, &format!("mobile.layer.keys.{kcmd}"), klbl);
                        ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
                        crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, klbl, Tokens::medium(11.0), kcw - 4.0 * u, t.text);
                        if resp.clicked() {
                            cmd = Some((kcmd, json!({})));
                        }
                    }
                    // Row 3: Hold / Linear / Graph Editor
                    let (more_row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                    let mores: [(&'static str, &'static str); 3] = [
                        ("keys.toggleHold", "Toggle Hold"),
                        ("keys.linear", "Linear"),
                        ("timeline.graphEditor", "📈 Graph Editor"),
                    ];
                    for (i, (kcmd, klbl)) in mores.iter().enumerate() {
                        let r = Rect::from_min_size(pos2(more_row.min.x + (kcw + gap) * i as f32, more_row.min.y), vec2(kcw, 42.0 * u));
                        let resp = kit::tap(app, ui, r, &format!("mobile.layer.keys.{kcmd}"), klbl);
                        ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
                        crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, klbl, Tokens::medium(11.0), kcw - 4.0 * u, t.text);
                        if resp.clicked() {
                            if *kcmd == "keys.linear" {
                                cmd = Some(("keys.interpolation", json!({"interpolation": "linear"})));
                            } else {
                                cmd = Some((kcmd, json!({})));
                            }
                        }
                    }
                }
            }

            // ─── Timing & Playhead ─────────────────────────────────────────
            kit::heading(app, ui, w, "Timing & Playhead", u);
            {
                if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Stopwatch), "In/Out & Work Area", Some(if timing_open { "▾" } else { "›" }), None, true, "mobile.layer.timing.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(timing_sub_id, !timing_open));
                }
                if timing_open {
                    let tcols = 2usize;
                    let tcw = (w - gap) / tcols as f32;
                    let timing_cmds: [(&'static str, &'static str); 6] = [
                        ("timeline.trimInToTime", "Trim In to Playhead ["),
                        ("timeline.trimOutToTime", "Trim Out to Playhead ]"),
                        ("timeline.moveInToTime", "Move In to Playhead"),
                        ("timeline.moveOutToTime", "Move Out to Playhead"),
                        ("timeline.workAreaBegin", "Set Work Area Start (B)"),
                        ("timeline.workAreaEnd", "Set Work Area End (N)"),
                    ];
                    for chunk in timing_cmds.chunks(tcols) {
                        let (row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                        for (i, (tcmd, tlbl)) in chunk.iter().enumerate() {
                            let r = Rect::from_min_size(pos2(row.min.x + (tcw + gap) * i as f32, row.min.y), vec2(tcw, 42.0 * u));
                            let resp = kit::tap(app, ui, r, &format!("mobile.layer.timing.{tcmd}"), tlbl);
                            ui.painter().rect_filled(r, 8.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
                            crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, tlbl, Tokens::medium(11.5), tcw - 6.0 * u, t.text);
                            if resp.clicked() {
                                cmd = Some((tcmd, json!({})));
                            }
                        }
                    }
                }
            }

            // ─── Label Color ───────────────────────────────────────────────
            kit::heading(app, ui, w, "Label Color", u);
            {
                let cur_label_name = LABEL_COLORS.iter().find(|(i, _, _)| *i == label_idx).map(|(_, n, _)| *n).unwrap_or("None");
                if kit::list_row(app, ui, w, 46.0 * u, None, &format!("Label: {cur_label_name}"), Some(if label_open { "▾" } else { "›" }), None, true, "mobile.layer.label.toggle", u) {
                    ctx.data_mut(|d| d.insert_temp(label_sub_id, !label_open));
                }
                if label_open {
                    let cols = 4usize;
                    let cw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
                    for chunk in LABEL_COLORS.chunks(cols) {
                        let (row, _) = ui.allocate_exact_size(vec2(w, 42.0 * u + gap), Sense::hover());
                        for (i, (idx, lbl, color)) in chunk.iter().enumerate() {
                            let r = Rect::from_min_size(pos2(row.min.x + (cw + gap) * i as f32, row.min.y), vec2(cw, 42.0 * u));
                            let resp = kit::tap(app, ui, r, &format!("mobile.layer.label.{idx}"), lbl);
                            let sel = *idx == label_idx;
                            ui.painter().rect_filled(r, 10.0 * u, *color);
                            if sel {
                                ui.painter().rect_stroke(r, 10.0 * u, egui::Stroke::new(2.5, Color32::WHITE), egui::StrokeKind::Inside);
                            }
                            if resp.is_pointer_button_down_on() {
                                ui.painter().rect_filled(r, 10.0 * u, Color32::from_white_alpha(60));
                            }
                            crate::widgets::text_fit(ui.painter(), r.center(), Align2::CENTER_CENTER, lbl, Tokens::medium(10.5), cw - 6.0 * u, Color32::WHITE);
                            if resp.clicked() {
                                cmd = Some(("layer.setSwitch", json!({"layers": [layer], "switch": "label", "value": idx})));
                                ctx.data_mut(|d| d.insert_temp(label_sub_id, false));
                            }
                        }
                    }
                }
            }

            // ─── Create ────────────────────────────────────────────────────
            kit::heading(app, ui, w, "Create", u);
            let cols = 3usize;
            let cw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
            let ch = 68.0 * u;
            for chunk in CREATE.chunks(cols) {
                let (row, _) = ui.allocate_exact_size(vec2(w, ch + gap), Sense::hover());
                for (i, (id, ic, label)) in chunk.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(row.min.x + (cw + gap) * i as f32, row.min.y), vec2(cw, ch));
                    let resp = kit::tap(app, ui, r, &format!("mobile.layer.new.{label}"), label);
                    ui.painter().rect_filled(r, 14.0 * u, if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
                    crate::icons::paint(ui.painter(), Rect::from_center_size(pos2(r.center().x, r.min.y + ch * 0.36), vec2(24.0 * u, 24.0 * u)), *ic, t.icon_active);
                    crate::widgets::text_fit(ui.painter(), pos2(r.center().x, r.max.y - 13.0 * u), Align2::CENTER_CENTER, label, Tokens::medium(11.5), cw - 8.0 * u, t.text);
                    if resp.clicked() {
                        cmd = Some((id, json!({})));
                    }
                }
            }

            // ─── Layer Operations ──────────────────────────────────────────
            kit::heading(app, ui, w, "Layer", u);
            // Rename row
            if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Pen), "Rename", None, None, true, "mobile.layer.op.rename", u) {
                cmd = Some(("layer.rename", json!({"layer": layer, "name": name})));
            }
            let ops: [(&'static str, Icon, &str); 4] =
                [("layer.precompose", Icon::Comp, "Pre-compose"), ("edit.duplicate", Icon::Duplicate, "Duplicate"), ("edit.clear", Icon::Trash, "Delete"), ("edit.splitLayer", Icon::Scissors, "Split Layer")];
            for (id, ic, label) in ops {
                if kit::list_row(app, ui, w, 46.0 * u, Some(ic), label, None, None, true, &format!("mobile.layer.op.{id}"), u) {
                    cmd = Some((id, json!({})));
                }
            }
            // Add Expression
            if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Fx), "Add Expression (Alt+Shift+=)", None, None, true, "mobile.layer.op.addExpr", u) {
                cmd = Some(("prop.setExpression", json!({})));
            }
            if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Keyframe), "Convert Expression to Keyframes", None, None, true, "mobile.layer.op.exprToKeys", u) {
                cmd = Some(("prop.convertExpressionToKeyframes", json!({})));
            }
            // Add Effect shortcut
            if kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Fx), "Add Effect", None, None, true, "mobile.layer.op.addEffect", u) {
                // Switch to the Effects & Presets panel via the + menu
                let _ = crate::menus::invoke(app, ctx, "window.panel", json!({"panel": "effects_presets"}));
                close = true;
            }
            if let Some(item) = comp_item
                && kit::list_row(app, ui, w, 46.0 * u, Some(Icon::Comp), "Open Composition", None, None, true, "mobile.layer.op.open", u)
            {
                open_comp = Some(item);
            }

            // ─── Settings Submenus ─────────────────────────────────────────
            kit::heading(app, ui, w, "Settings", u);
            for (key, label) in GROUPS {
                if kit::list_row(app, ui, w, 46.0 * u, None, label, Some("›"), None, !layer_sub(key).is_empty(), &format!("mobile.layer.group.{key}"), u) {
                    ui.data_mut(|d| d.insert_temp(sub_id, key.to_string()));
                }
            }
            if kit::list_row(app, ui, w, 46.0 * u, None, "Layer Settings…", None, None, true, "mobile.layer.settings", u) {
                cmd = Some(("layer.settings", json!({})));
            }

            // ─── Switches ──────────────────────────────────────────────────
            kit::heading(app, ui, w, "Switches", u);
            let cols = 2usize;
            let cw = (w - gap) / cols as f32;
            for (ci, chunk) in SWITCHES.chunks(cols).enumerate() {
                let (row, _) = ui.allocate_exact_size(vec2(w, 46.0 * u + gap), Sense::hover());
                for (i, (key, label)) in chunk.iter().enumerate() {
                    let on = switches[ci * cols + i];
                    let r = Rect::from_min_size(pos2(row.min.x + (cw + gap) * i as f32, row.min.y), vec2(cw, 46.0 * u));
                    let resp = kit::tap(app, ui, r, &format!("mobile.layer.switch.{key}"), label);
                    let fill = if on { t.accent } else if resp.is_pointer_button_down_on() { t.pressed } else { t.hover };
                    ui.painter().rect_filled(r, r.height() / 2.0, fill);
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, label, Tokens::medium(13.0), if on { Color32::WHITE } else { t.text });
                    if resp.clicked() {
                        cmd = Some(("layer.setSwitch", json!({"layers": [layer], "switch": key})));
                    }
                }
            }
            ui.add_space(16.0 * u);
        });
    });

    if back {
        ctx.data_mut(|d| d.remove::<String>(sub_id));
        ctx.data_mut(|d| d.insert_temp(parent_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(label_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(blend_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(matte_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(styles_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(timing_sub_id, false));
        ctx.data_mut(|d| d.insert_temp(keys_sub_id, false));
    }
    if let Some(item) = open_comp {
        let _ = app.session.execute("comp.open", json!({"comp": item}));
        close = true;
    }
    if let Some((id, params)) = cmd {
        let before_sub = sub.is_some();
        match id {
            "layer.settings" | "layer.precompose" | "edit.splitLayer" | "edit.duplicate" | "edit.clear" => {
                let res = menus::invoke(app, ctx, id, params);
                if let Err(e) = res {
                    app.ui.status = e;
                }
                close = true;
            }
            "layer.setSwitch" | "layer.setParent" | "layer.setBlendMode" | "layer.trackMatte" => {
                if let Err(e) = app.session.execute(id, params) {
                    app.ui.status = e.to_string();
                }
            }
            _ => {
                // Timing, keyframe, style, and creation commands
                let res = if id.starts_with("timeline.") || id.starts_with("app.") || id.starts_with("file.") {
                    menus::invoke(app, ctx, id, params)
                } else if let Err(_e) = app.session.execute(id, params) {
                    menus::invoke(app, ctx, id, json!({}))
                } else {
                    Ok(json!({}))
                };
                if let Err(e) = res {
                    app.ui.status = e;
                }
                // Creating a layer is a finished action (the new layer is selected): the sheet closes;
                // toggles, parent, blending and the like keep it open for the next change.
                if id.starts_with("layer.new") {
                    close = true;
                }
            }
        }
        let _ = before_sub;
    }
    if chosen.is_some() {
        run(app, ctx, chosen);
        close = true;
    }
    if close || dismissed {
        ctx.data_mut(|d| d.remove::<String>(sub_id));
        ctx.data_mut(|d| d.remove::<bool>(parent_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(label_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(blend_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(matte_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(styles_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(timing_sub_id));
        ctx.data_mut(|d| d.remove::<bool>(keys_sub_id));
        return true;
    }
    false
}

/// The Home page's button style, reused for the sheet's dismiss actions.
pub fn _button_style() -> Btn {
    Btn::Secondary
}
