//! Touch widgets shared by the phone UI: buttons with a pressed state, horizontally scrolling
//! strips, popups that stay on screen, bottom sheets and list rows. Sizes are in dp-like units
//! (`u` points per dp), so a 44 here is a 44 dp touch target at any UI scale.

use egui::{Align2, Color32, CornerRadius, Id, Mesh, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2};

use crate::icons::Icon;
use crate::theme::Tokens;
use crate::{EffectcraftApp, widgets};

/// Points per dp: the UI scale and the screen density folded into one number.
pub fn dp(ctx: &egui::Context) -> f32 {
    let ppp = ctx.pixels_per_point().max(0.1);
    (ctx.native_pixels_per_point().unwrap_or(ppp) / ppp).clamp(0.5, 2.5)
}

/// A tap target registered with the automation registry (so tests and scripts can press it).
pub fn tap(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, id: &str, label: &str) -> Response {
    app.auto.add(id, rect, label);
    ui.interact(rect, Id::new(("m", id)), Sense::click())
}

/// A flat icon button with a pressed state (a finger has no hover). Returns whether it was tapped.
#[allow(clippy::too_many_arguments)]
pub fn icon_tap(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, icon: Icon, label: &str, on: bool, enabled: bool, id: &str, u: f32) -> bool {
    let t = app.tokens;
    let resp = tap(app, ui, rect, id, label);
    let inner = rect.shrink2(vec2(3.0 * u, 5.0 * u));
    if on {
        ui.painter().rect_filled(inner, 10.0 * u, t.accent);
    } else if enabled && resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(inner, 10.0 * u, t.pressed);
    }
    let col = if !enabled {
        t.text_faint
    } else if on {
        Color32::WHITE
    } else {
        t.icon_active
    };
    let s = 22.0 * u;
    crate::icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(s, s)), icon, col);
    enabled && resp.clicked()
}

#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// A pill button with a text label. Returns whether it was tapped.
#[allow(clippy::too_many_arguments)]
pub fn button(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, label: &str, kind: Btn, enabled: bool, id: &str, u: f32) -> bool {
    let t = app.tokens;
    let resp = tap(app, ui, rect, id, label);
    let down = enabled && resp.is_pointer_button_down_on();
    let (fill, fg) = match (kind, enabled) {
        (_, false) => (t.field_bg, t.text_faint),
        (Btn::Primary, _) => (if down { t.accent_hover } else { t.accent }, Color32::WHITE),
        (Btn::Secondary, _) => (if down { t.pressed } else { t.hover }, t.text),
        (Btn::Ghost, _) => (if down { t.pressed } else { Color32::TRANSPARENT }, t.accent),
        (Btn::Danger, _) => (if down { Color32::from_rgb(0xb0, 0x3a, 0x3a) } else { Color32::from_rgb(0xc9, 0x45, 0x45) }, Color32::WHITE),
    };
    ui.painter().rect_filled(rect, rect.height() / 2.0, fill);
    if kind == Btn::Ghost {
        ui.painter().rect_stroke(rect, rect.height() / 2.0, Stroke::new(1.5, if enabled { t.accent } else { t.text_faint }), egui::StrokeKind::Inside);
    }
    widgets::text_fit(ui.painter(), rect.center(), Align2::CENTER_CENTER, label, Tokens::semibold(14.0), rect.width() - 16.0 * u, fg);
    enabled && resp.clicked()
}

/// One entry of a scrolling strip.
#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub icon: Option<Icon>,
    pub text: String,
    pub on: bool,
    pub enabled: bool,
    pub w: f32,
}

impl Item {
    pub fn icon(id: &str, icon: Icon, w: f32) -> Item {
        Item { id: id.to_string(), icon: Some(icon), text: id.to_string(), on: false, enabled: true, w }
    }
    pub fn text(id: &str, text: &str, w: f32) -> Item {
        Item { id: id.to_string(), icon: None, text: text.to_string(), on: false, enabled: true, w }
    }
    pub fn on(mut self, on: bool) -> Item {
        self.on = on;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Item {
        self.enabled = enabled;
        self
    }
}

pub struct StripOut {
    pub clicked: Option<usize>,
    /// On-screen rectangles of the items (for anchoring popups), whatever the scroll offset.
    pub rects: Vec<Rect>,
}

fn fade(p: &egui::Painter, r: Rect, to_right: bool, c: Color32) {
    let (a, b) = if to_right { (Color32::TRANSPARENT, c) } else { (c, Color32::TRANSPARENT) };
    let mut m = Mesh::default();
    m.colored_vertex(r.left_top(), a);
    m.colored_vertex(r.right_top(), b);
    m.colored_vertex(r.right_bottom(), b);
    m.colored_vertex(r.left_bottom(), a);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

/// A horizontally scrolling row of icon / text buttons, with a fade at the edges that can scroll.
pub fn strip(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, salt: &str, items: &[Item], u: f32) -> StripOut {
    let t = app.tokens;
    let mut out = StripOut { clicked: None, rects: Vec::new() };
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).id_salt(("m-strip", salt)));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    let h = rect.height();
    let sa = egui::ScrollArea::horizontal()
        .id_salt(("m-strip-scroll", salt))
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut child, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.horizontal(|ui| {
                for (i, it) in items.iter().enumerate() {
                    let (r, _) = ui.allocate_exact_size(vec2(it.w, h), Sense::hover());
                    out.rects.push(r);
                    let resp = ui.interact(r, Id::new(("m-strip-item", salt, it.id.as_str())), Sense::click());
                    app.auto.add(&it.id, r, &it.text);
                    let down = it.enabled && resp.is_pointer_button_down_on();
                    let inner = r.shrink2(vec2(2.0 * u, 5.0 * u));
                    let p = ui.painter();
                    if let Some(icon) = it.icon {
                        if it.on {
                            p.rect_filled(inner, 10.0 * u, t.accent);
                        } else if down {
                            p.rect_filled(inner, 10.0 * u, t.pressed);
                        }
                        let col = if !it.enabled { t.text_faint } else if it.on { Color32::WHITE } else { t.icon_active };
                        crate::icons::paint(p, Rect::from_center_size(r.center(), vec2(22.0 * u, 22.0 * u)), icon, col);
                    } else {
                        if down {
                            p.rect_filled(inner, 10.0 * u, t.pressed);
                        }
                        let font = if it.on { Tokens::semibold(13.5) } else { Tokens::medium(13.5) };
                        let col = if !it.enabled { t.text_faint } else if it.on { t.text } else { t.text_dim };
                        p.text(r.center() - vec2(0.0, 1.0 * u), Align2::CENTER_CENTER, &it.text, font, col);
                        if it.on {
                            p.rect_filled(Rect::from_min_max(pos2(r.min.x + 8.0 * u, r.max.y - 3.0 * u), pos2(r.max.x - 8.0 * u, r.max.y)), CornerRadius::same(2), t.accent);
                        }
                    }
                    if resp.clicked() && it.enabled {
                        out.clicked = Some(i);
                    }
                }
            });
        });
    let max_off = (sa.content_size.x - sa.inner_rect.width()).max(0.0);
    let p = ui.painter();
    let fw = 18.0 * u;
    if sa.state.offset.x > 1.0 {
        fade(p, Rect::from_min_size(rect.min, vec2(fw, h)), false, t.header_bg);
    }
    if sa.state.offset.x < max_off - 1.0 {
        fade(p, Rect::from_min_max(pos2(rect.max.x - fw, rect.min.y), rect.max), true, t.header_bg);
    }
    out
}

/// Run `add` in an Area on the foreground layer covering `rect`.
pub fn overlay(ctx: &egui::Context, id: &str, rect: Rect, add: impl FnOnce(&mut Ui)) {
    egui::Area::new(Id::new(id)).order(egui::Order::Foreground).fixed_pos(rect.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(rect);
        let (_, _) = ui.allocate_exact_size(rect.size(), Sense::hover());
        add(ui);
    });
}

/// A full-bounds layer that swallows taps; returns true when it was tapped (dismiss).
pub fn scrim(ctx: &egui::Context, id: &str, bounds: Rect, alpha: u8) -> bool {
    let mut tapped = false;
    egui::Area::new(Id::new(id)).order(egui::Order::Foreground).fixed_pos(bounds.min).constrain(false).show(ctx, |ui| {
        let (r, resp) = ui.allocate_exact_size(bounds.size(), Sense::click());
        if alpha > 0 {
            ui.painter().rect_filled(r, 0.0, Color32::from_black_alpha(alpha));
        }
        tapped = resp.clicked();
    });
    tapped
}

/// Where a popup of `size` goes next to `anchor`: below it (or above when there is no room),
/// centred on it, and always inside `bounds`.
pub fn popup_rect(anchor: Rect, size: Vec2, bounds: Rect, u: f32) -> Rect {
    let m = 8.0 * u;
    let avail_w = (bounds.width() - 2.0 * m).max(40.0);
    let w = size.x.min(avail_w);
    let below = bounds.max.y - anchor.max.y - m;
    let above = anchor.min.y - bounds.min.y - m;
    let (h, y) = if size.y <= below || below >= above {
        let h = size.y.min(below.max(60.0));
        (h, anchor.max.y + 4.0 * u)
    } else {
        let h = size.y.min(above.max(60.0));
        (h, anchor.min.y - 4.0 * u - h)
    };
    let x = (anchor.center().x - w / 2.0).clamp(bounds.min.x + m, (bounds.max.x - m - w).max(bounds.min.x + m));
    Rect::from_min_size(pos2(x, y.clamp(bounds.min.y, (bounds.max.y - h).max(bounds.min.y))), vec2(w, h))
}

/// A popup next to `anchor`; a tap outside closes it. `add` draws into the popup's inner
/// rectangle. Returns true when it should close.
#[allow(clippy::too_many_arguments)]
pub fn popup(t: Tokens, ctx: &egui::Context, id: &str, anchor: Rect, bounds: Rect, size: Vec2, u: f32, add: impl FnOnce(&mut Ui, Rect)) -> bool {
    let dismissed = scrim(ctx, &format!("{id}-scrim"), bounds, 0);
    let r = popup_rect(anchor, size, bounds, u);
    overlay(ctx, id, r, |ui| {
        ui.painter().rect_filled(r.translate(vec2(0.0, 2.0 * u)), 14.0 * u, Color32::from_black_alpha(90));
        ui.painter().rect_filled(r, 14.0 * u, t.panel_bg);
        ui.painter().rect_stroke(r, 14.0 * u, Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
        add(ui, r.shrink(6.0 * u));
    });
    dismissed
}

/// A bottom sheet of `height` with a grab bar; a tap on the scrim closes it. `add` draws into
/// the sheet's inner rectangle. Returns true when it should close.
pub fn sheet(t: Tokens, ctx: &egui::Context, id: &str, bounds: Rect, height: f32, u: f32, add: impl FnOnce(&mut Ui, Rect)) -> bool {
    let dismissed = scrim(ctx, &format!("{id}-scrim"), bounds, 140);
    let h = height.min(bounds.height());
    let r = Rect::from_min_size(pos2(bounds.min.x, bounds.max.y - h), vec2(bounds.width(), h));
    overlay(ctx, id, r, |ui| {
        let top = (18.0 * u) as u8;
        ui.painter().rect_filled(r, CornerRadius { nw: top, ne: top, sw: 0, se: 0 }, t.panel_bg);
        ui.painter().rect_filled(Rect::from_center_size(pos2(r.center().x, r.min.y + 9.0 * u), vec2(36.0 * u, 4.0 * u)), CornerRadius::same(2), t.text_faint);
        add(ui, Rect::from_min_max(pos2(r.min.x + 12.0 * u, r.min.y + 22.0 * u), pos2(r.max.x - 12.0 * u, r.max.y - 8.0 * u)));
    });
    dismissed
}

/// A list row: optional icon, label, optional right-hand detail, optional check. Returns taps.
#[allow(clippy::too_many_arguments)]
pub fn list_row(app: &mut EffectcraftApp, ui: &mut Ui, width: f32, h: f32, icon: Option<Icon>, label: &str, detail: Option<&str>, checked: Option<bool>, enabled: bool, id: &str, u: f32) -> bool {
    let t = app.tokens;
    let (r, _) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    let resp = tap(app, ui, r, id, label);
    let p = ui.painter();
    if enabled && resp.is_pointer_button_down_on() {
        p.rect_filled(r, 10.0 * u, t.pressed);
    }
    let col = if enabled { t.text } else { t.text_faint };
    let mut x = r.min.x + 12.0 * u;
    if let Some(icon) = icon {
        crate::icons::paint(p, Rect::from_center_size(pos2(x + 11.0 * u, r.center().y), vec2(22.0 * u, 22.0 * u)), icon, if enabled { t.icon_active } else { t.text_faint });
        x += 34.0 * u;
    } else if checked.is_some() {
        if checked == Some(true) {
            let c = pos2(x + 8.0 * u, r.center().y);
            let s = Stroke::new(2.2, t.accent);
            p.line_segment([c + vec2(-5.0 * u, 0.0), c + vec2(-1.5 * u, 4.0 * u)], s);
            p.line_segment([c + vec2(-1.5 * u, 4.0 * u), c + vec2(5.5 * u, -4.5 * u)], s);
        }
        x += 28.0 * u;
    }
    let right = detail.map(|d| p.layout_no_wrap(d.to_string(), Tokens::ui(12.0), t.text_faint).size().x + 16.0 * u).unwrap_or(0.0);
    widgets::text_fit(p, pos2(x, r.center().y), Align2::LEFT_CENTER, label, Tokens::medium(14.0), (r.max.x - x - right - 8.0 * u).max(20.0), col);
    if let Some(d) = detail {
        p.text(pos2(r.max.x - 12.0 * u, r.center().y), Align2::RIGHT_CENTER, d, Tokens::ui(12.0), t.text_faint);
    }
    enabled && resp.clicked()
}

/// A small caption above a group of rows.
pub fn heading(app: &EffectcraftApp, ui: &mut Ui, width: f32, text: &str, u: f32) {
    let t = app.tokens;
    let (r, _) = ui.allocate_exact_size(vec2(width, 30.0 * u), Sense::hover());
    ui.painter().text(pos2(r.min.x + 6.0 * u, r.max.y - 8.0 * u), Align2::LEFT_CENTER, text, Tokens::semibold(12.0), t.text_dim);
}
