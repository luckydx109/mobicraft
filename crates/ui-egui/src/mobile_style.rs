//! The phone's Fill & Stroke panel: the PC header's tool options (Fill, Stroke, Stroke Width,
//! Tool Creates Shape / Mask, Snapping) as touch cards in the + panel. It drives the same engine
//! commands and state as the PC header (`shape.toolOptions`, `shape.fillStroke`,
//! `session.state.shape_tool`), so a shape drawn here looks the same as one drawn on a PC.
use egui::{Align2, Color32, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use effectcraft_color::BlendMode;
use effectcraft_engine::commands::shape_tool::{PaintKind, ToolPaint};
use effectcraft_engine::project::{LayerId, LayerSource};

use crate::mobile_kit::{self as kit, Btn};
use crate::theme::Tokens;
use crate::{EffectcraftApp, widgets};

fn picker_id(key: &str) -> Id {
    Id::new(("m-style-pick", key))
}

fn tool_options(app: &mut EffectcraftApp, v: serde_json::Value) {
    if let Err(e) = app.session.execute("shape.toolOptions", v) {
        app.ui.status = e.to_string();
    }
}

/// The selected shape layers' paint is shown and edited too (as in After Effects).
fn shape_layers(app: &mut EffectcraftApp) -> Vec<u64> {
    let layers: Vec<u64> = app
        .session
        .active_comp()
        .map(|c| app.session.state.selected_layers.iter().filter(|id| c.layer(**id).is_some_and(|l| matches!(l.source, LayerSource::Shape))).map(|id| id.0).collect())
        .unwrap_or_default();
    let layer_paint = layers.first().and_then(|id| app.session.active_comp()?.layer(LayerId(*id))).map(|l| {
        let lt = l.layer_time(app.session.time());
        (effectcraft_engine::commands::shape_stroke::first_paint(l, lt), effectcraft_engine::commands::shape_stroke::first_paint_kinds(l, lt))
    });
    if let Some(((fill, stroke, width), (fill_kind, stroke_kind))) = layer_paint {
        let rgb = |c: [f64; 4]| [c[0] as f32, c[1] as f32, c[2] as f32];
        let tool = &mut app.session.state.shape_tool;
        tool.fill.kind = fill_kind;
        tool.stroke.kind = stroke_kind;
        if let Some(c) = fill {
            tool.fill.color = rgb(c);
        }
        if let Some(c) = stroke {
            tool.stroke.color = rgb(c);
        }
        tool.stroke_width = width.unwrap_or(0.0);
    }
    layers
}

/// Apply a paint value to the selected shape layers, one undo step per drag or picker session.
fn apply(app: &mut EffectcraftApp, layers: &[u64], key: &str, v: serde_json::Value) {
    if layers.is_empty() {
        return;
    }
    if let Err(e) = app.session.execute("shape.fillStroke", json!({"layers": layers, key: v, "merge": format!("tool-{key}")})) {
        app.ui.status = e.to_string();
    }
}

/// A Fill or Stroke kind: for new shapes and the selected shape layers (None switches them off).
fn paint_kind(app: &mut EffectcraftApp, layers: &[u64], key: &str, kind: PaintKind) {
    let k_type = format!("{key}Type");
    tool_options(app, json!({&k_type: kind.id()}));
    if layers.is_empty() {
        return;
    }
    let mut v = json!({"layers": layers, k_type: kind.id()});
    let color = if key == "fill" { app.session.state.shape_tool.fill.color } else { app.session.state.shape_tool.stroke.color };
    if let (PaintKind::Solid, Some(o)) = (kind, v.as_object_mut()) {
        o.insert(key.to_string(), json!(color));
    }
    if let Err(e) = app.session.execute("shape.fillStroke", v) {
        app.ui.status = e.to_string();
    }
}

fn paint_swatch(p: &egui::Painter, r: Rect, paint: &ToolPaint, t: &Tokens, u: f32) {
    let clip = p.with_clip_rect(r);
    match paint.kind {
        PaintKind::None => {
            clip.rect_filled(r, 6.0 * u, Color32::WHITE);
            clip.line_segment([r.left_bottom(), r.right_top()], Stroke::new(2.5 * u, Color32::from_rgb(0xe0, 0x30, 0x30)));
        }
        PaintKind::Solid => {
            clip.rect_filled(r, 6.0 * u, widgets::srgb32(paint.color));
        }
        PaintKind::Linear => {
            let mut m = egui::Mesh::default();
            for (q, c) in [(r.left_top(), Color32::WHITE), (r.right_top(), Color32::BLACK), (r.right_bottom(), Color32::BLACK), (r.left_bottom(), Color32::WHITE)] {
                m.colored_vertex(q, c);
            }
            m.add_triangle(0, 1, 2);
            m.add_triangle(0, 2, 3);
            clip.add(m);
        }
        PaintKind::Radial => {
            let rad = r.size().length() / 2.0;
            for i in 0..8 {
                let f = 1.0 - i as f32 / 8.0;
                clip.circle_filled(r.center(), rad * f, Color32::from_gray((255.0 * (1.0 - f)) as u8));
            }
        }
    }
    p.rect_stroke(r, 6.0 * u, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
}

fn bytes(c: [f32; 3]) -> [u8; 3] {
    let c = widgets::srgb32(c);
    [c.r(), c.g(), c.b()]
}

/// `label  [-]  value  [+]`; returns -1, 0 or +1.
#[allow(clippy::too_many_arguments)]
fn stepper(app: &mut EffectcraftApp, ui: &mut Ui, row: Rect, label: &str, value: &str, id: &str, u: f32) -> i32 {
    let t = app.tokens;
    let p = ui.painter().clone();
    p.text(pos2(row.min.x + 14.0 * u, row.center().y), Align2::LEFT_CENTER, label, Tokens::ui(13.0), t.text_dim);
    let bw = 48.0 * u;
    let plus = Rect::from_min_size(pos2(row.max.x - 12.0 * u - bw, row.min.y + 4.0 * u), vec2(bw, row.height() - 8.0 * u));
    let minus = Rect::from_min_size(pos2(plus.min.x - 96.0 * u - 8.0 * u, plus.min.y), vec2(bw, plus.height()));
    p.text(pos2((minus.max.x + plus.min.x) / 2.0, row.center().y), Align2::CENTER_CENTER, value, Tokens::semibold(14.0), t.text);
    let mut d = 0;
    if kit::button(app, ui, minus, "-", Btn::Secondary, true, &format!("{id}.dec"), u) {
        d = -1;
    }
    if kit::button(app, ui, plus, "+", Btn::Secondary, true, &format!("{id}.inc"), u) {
        d = 1;
    }
    d
}

/// The panel: `rect` is the + panel's body.
pub fn show(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, u: f32) {
    let t = app.tokens;
    let ctx = ui.ctx().clone();
    ui.painter().rect_filled(rect, 0.0, t.panel_bg);
    let layers = shape_layers(app);
    // One undo step per picker session or drag: end the merge once the finger is up.
    let picking = ["fill", "stroke"].iter().any(|k| widgets::popup_is_open(ui, picker_id(k)));
    if !picking && !ui.input(|i| i.pointer.any_down()) && app.session.history.merge_key.as_deref().is_some_and(|k| k.starts_with("tool-")) {
        app.session.history.merge_key = None;
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).id_salt("m-style"));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    egui::ScrollArea::vertical().id_salt("m-style-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
        let w = rect.width();
        let row = |ui: &mut Ui, h: f32| ui.allocate_exact_size(vec2(w, h * u), Sense::hover()).0;
        let p = ui.painter().clone();

        let r = row(ui, 44.0);
        let note = if layers.is_empty() { "New shapes get this fill and stroke.".to_string() } else { format!("New shapes and the {} selected shape layer(s).", layers.len()) };
        p.text(pos2(r.min.x + 14.0 * u, r.center().y), Align2::LEFT_CENTER, note, Tokens::ui(12.0), t.text_dim);
        app.auto.add("mobile.style.note", r, "Fill & Stroke");

        for (key, title) in [("fill", "Fill"), ("stroke", "Stroke")] {
            let tp = if key == "fill" { app.session.state.shape_tool.fill.clone() } else { app.session.state.shape_tool.stroke.clone() };
            // Title row with the current kind.
            let r = row(ui, 38.0);
            p.text(pos2(r.min.x + 14.0 * u, r.center().y), Align2::LEFT_CENTER, title, Tokens::semibold(16.0), t.text);
            p.text(pos2(r.max.x - 14.0 * u, r.center().y), Align2::RIGHT_CENTER, tp.kind.label(), Tokens::ui(12.0), t.text_dim);
            // None / Solid Color / Linear Gradient / Radial Gradient.
            let r = row(ui, 56.0);
            let gap = 8.0 * u;
            let cw = (r.width() - 24.0 * u - gap * 3.0) / 4.0;
            for (i, k) in PaintKind::ALL.into_iter().enumerate() {
                let b = Rect::from_min_size(pos2(r.min.x + 12.0 * u + i as f32 * (cw + gap), r.min.y + 4.0 * u), vec2(cw, r.height() - 8.0 * u));
                let label = match k {
                    PaintKind::None => "None",
                    PaintKind::Solid => "Solid",
                    PaintKind::Linear => "Linear",
                    PaintKind::Radial => "Radial",
                };
                if kit::button(app, ui, b, label, if k == tp.kind { Btn::Primary } else { Btn::Secondary }, true, &format!("mobile.style.{key}.kind.{}", k.id()), u) {
                    paint_kind(app, &layers, key, k);
                }
            }
            // The colour: a big swatch and its values.
            let r = row(ui, 76.0);
            let sw = Rect::from_min_size(pos2(r.min.x + 14.0 * u, r.min.y + 6.0 * u), vec2(64.0 * u, r.height() - 12.0 * u));
            let sresp = kit::tap(app, ui, sw, &format!("mobile.style.{key}.swatch"), &format!("{title} Color"));
            paint_swatch(&p, sw, &tp, &t, u);
            let [rr, gg, bb] = bytes(tp.color);
            let (line1, line2) = match tp.kind {
                PaintKind::None => ("No ".to_string() + &title.to_lowercase(), "Pick Solid, Linear or Radial".to_string()),
                PaintKind::Solid => (format!("#{rr:02X}{gg:02X}{bb:02X}"), format!("R {rr}   G {gg}   B {bb}")),
                _ => (tp.kind.label().to_string(), "Tap the swatch to edit the colors".to_string()),
            };
            p.text(pos2(sw.max.x + 14.0 * u, r.center().y - 11.0 * u), Align2::LEFT_CENTER, line1, Tokens::semibold(15.0), t.text);
            p.text(pos2(sw.max.x + 14.0 * u, r.center().y + 11.0 * u), Align2::LEFT_CENTER, line2, Tokens::ui(12.0), t.text_dim);
            if sresp.clicked() {
                match tp.kind {
                    PaintKind::Solid => widgets::open_popup(ui, picker_id(key)),
                    PaintKind::Linear | PaintKind::Radial => {
                        let gid = if key == "fill" { "gfill" } else { "gstroke" };
                        if let Some((layer, prop)) = crate::header::gradient_colors(app, &layers, gid) {
                            if let Err(e) = crate::menus::invoke(app, &ctx, "prop.editGradient", json!({"layer": layer, "prop": prop})) {
                                app.ui.status = e;
                            }
                        } else {
                            app.ui.status = "Select a shape layer with that gradient to edit its colors".into();
                        }
                    }
                    PaintKind::None => paint_kind(app, &layers, key, PaintKind::Solid),
                }
            }
            let mut c = tp.color;
            if widgets::color_popup(ui, picker_id(key), sw.left_bottom(), &mut c) {
                tool_options(app, json!({key: c}));
                apply(app, &layers, key, json!(c));
            }
            // Opacity (new shapes), blend mode.
            let r = row(ui, 56.0);
            let d = stepper(app, ui, r, "Opacity", &format!("{:.0}%", tp.opacity), &format!("mobile.style.{key}.opacity"), u);
            if d != 0 {
                let v = (tp.opacity + 5.0 * d as f64).clamp(0.0, 100.0);
                tool_options(app, json!({format!("{key}Opacity"): v}));
            }
            let r = row(ui, 52.0);
            let br = Rect::from_min_size(pos2(r.min.x + 12.0 * u, r.min.y + 4.0 * u), vec2(r.width() - 24.0 * u, r.height() - 8.0 * u));
            if kit::button(app, ui, br, &format!("Blend Mode: {}", tp.blend.label()), Btn::Secondary, true, &format!("mobile.style.{key}.blend"), u) {
                ctx.data_mut(|d| d.insert_temp(Id::new("m-style-blend"), key.to_string()));
            }
            let r = row(ui, 14.0);
            p.line_segment([pos2(r.min.x + 12.0 * u, r.center().y), pos2(r.max.x - 12.0 * u, r.center().y)], Stroke::new(1.0, t.separator));
        }

        // Stroke width.
        let r = row(ui, 56.0);
        let width = app.session.state.shape_tool.stroke_width;
        let d = stepper(app, ui, r, "Stroke Width", &format!("{width:.1} px"), "mobile.style.strokeWidth", u);
        if d != 0 {
            let v = (width + 0.5 * d as f64).clamp(0.0, 1000.0);
            tool_options(app, json!({"strokeWidth": v}));
            apply(app, &layers, "strokeWidth", json!(v));
        }
        let r = row(ui, 52.0);
        let sizes = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
        let cw = (r.width() - 24.0 * u - 8.0 * u * (sizes.len() as f32 - 1.0)) / sizes.len() as f32;
        for (i, v) in sizes.into_iter().enumerate() {
            let b = Rect::from_min_size(pos2(r.min.x + 12.0 * u + i as f32 * (cw + 8.0 * u), r.min.y + 4.0 * u), vec2(cw, r.height() - 8.0 * u));
            if kit::button(app, ui, b, &format!("{v:.0}"), if (width - v).abs() < 0.01 { Btn::Primary } else { Btn::Secondary }, true, &format!("mobile.style.strokeWidth.{v:.0}"), u) {
                tool_options(app, json!({"strokeWidth": v}));
                apply(app, &layers, "strokeWidth", json!(v));
            }
        }

        // Tool Creates Shape / Mask, with a shape layer selected.
        if !layers.is_empty() {
            let r = row(ui, 56.0);
            let masks = app.session.state.shape_tool.creates_mask;
            let cw = (r.width() - 24.0 * u - 8.0 * u) / 2.0;
            for (i, (label, mask)) in [("Tool Creates Shape", false), ("Tool Creates Mask", true)].into_iter().enumerate() {
                let b = Rect::from_min_size(pos2(r.min.x + 12.0 * u + i as f32 * (cw + 8.0 * u), r.min.y + 4.0 * u), vec2(cw, r.height() - 8.0 * u));
                if kit::button(app, ui, b, label, if masks == mask { Btn::Primary } else { Btn::Secondary }, true, &format!("mobile.style.creates.{}", if mask { "mask" } else { "shape" }), u) {
                    tool_options(app, json!({"createsMask": mask}));
                }
            }
        }
        // Snapping.
        let r = row(ui, 56.0);
        let on = app.session.state.snapping;
        let b = Rect::from_min_size(pos2(r.min.x + 12.0 * u, r.min.y + 4.0 * u), vec2(r.width() - 24.0 * u, r.height() - 8.0 * u));
        if kit::button(app, ui, b, if on { "Snapping: On" } else { "Snapping: Off" }, if on { Btn::Primary } else { Btn::Secondary }, true, "mobile.style.snapping", u) {
            let _ = app.session.execute("view.snapping", json!({}));
        }
        row(ui, 24.0);
    });

    // The blend modes, over the panel.
    if let Some(key) = ctx.data(|d| d.get_temp::<String>(Id::new("m-style-blend"))) {
        let tp = if key == "fill" { app.session.state.shape_tool.fill.clone() } else { app.session.state.shape_tool.stroke.clone() };
        let bounds = ctx.content_rect();
        let size = vec2((bounds.width() - 32.0 * u).min(320.0 * u), (bounds.height() * 0.7).min(600.0 * u));
        let anchor = Rect::from_center_size(bounds.center(), vec2(1.0, 1.0));
        let mut pick: Option<BlendMode> = None;
        let mut close = kit::popup(app.tokens, &ctx, "m-style-blend-pop", anchor, bounds, size, u, |ui, inner| {
            let mut c = ui.new_child(egui::UiBuilder::new().max_rect(inner).id_salt("m-style-blend-list"));
            c.set_clip_rect(inner);
            egui::ScrollArea::vertical().id_salt("m-style-blend-scroll").auto_shrink([false, false]).show(&mut c, |ui| {
                for (i, m) in BlendMode::ALL.iter().enumerate() {
                    if kit::list_row(app, ui, inner.width(), 44.0 * u, None, m.label(), None, Some(*m == tp.blend), true, &format!("mobile.style.blend.{i}"), u) {
                        pick = Some(*m);
                    }
                }
            });
        });
        if let Some(m) = pick {
            tool_options(app, json!({format!("{key}Blend"): m.label()}));
            close = true;
        }
        if close {
            ctx.data_mut(|d| {
                d.remove::<String>(Id::new("m-style-blend"));
            });
        }
    }
}
