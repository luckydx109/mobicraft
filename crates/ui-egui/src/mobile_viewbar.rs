//! The preview's control strip for phones: a thin, horizontally scrolling row under the
//! composition viewer. It mirrors the desktop viewer's bottom bar (After Effects order) and drives
//! the same state and `view.*` commands; each control that has options opens a small popup that
//! stays on screen and closes on a tap outside. Controls that cannot work for the open
//! composition (3D views without a 3D layer, Show Snapshot without a snapshot) are dimmed.

use effectcraft_engine::commands::viewer_cmds::FastPreviews;
use effectcraft_engine::render::three_d::View3D;
use effectcraft_engine::viewer::Channel;
use egui::{Id, Rect, Ui, vec2};
use serde_json::json;

use crate::icons::Icon;
use crate::mobile_kit::{self as kit, Item};
use crate::panels::{viewer, viewer_tools as vt};
use crate::state::Resolution;
use crate::{EffectcraftApp, menus};

#[derive(Clone, Copy, PartialEq)]
enum Pop {
    Mag,
    Res,
    Grid,
    Channel,
    Exposure,
    Fast,
    View3D,
    Layout,
}

fn key() -> Id {
    Id::new("m-viewbar-popup")
}

const MAGS: [&str; 14] = ["Fit", "Fit up to 100%", "6.25%", "12.5%", "25%", "33.3%", "50%", "75%", "100%", "200%", "400%", "800%", "1600%", "3200%"];

fn text_w(ui: &Ui, s: &str, u: f32) -> f32 {
    ui.painter().layout_no_wrap(s.to_string(), crate::theme::Tokens::medium(13.5), egui::Color32::WHITE).size().x + 26.0 * u
}

fn run(app: &mut EffectcraftApp, ctx: &egui::Context, id: &str) {
    if let Err(e) = menus::invoke(app, ctx, id, json!({})) {
        app.ui.status = e;
    }
}

/// One option row of a popup: (label, checked).
fn choices(app: &mut EffectcraftApp, ui: &mut Ui, inner: Rect, rows: &[(String, bool)], prefix: &str, u: f32) -> Option<usize> {
    let mut hit = None;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).id_salt(("m-vb-list", prefix)));
    child.set_clip_rect(inner);
    egui::ScrollArea::vertical().id_salt(("m-vb-scroll", prefix)).auto_shrink([false, false]).show(&mut child, |ui| {
        for (i, (label, on)) in rows.iter().enumerate() {
            if kit::list_row(app, ui, inner.width(), 44.0 * u, None, label, None, Some(*on), true, &format!("mobile.view.{prefix}.{i}"), u) {
                hit = Some(i);
            }
        }
    });
    hit
}

/// Draw the strip in `rect` (the popups use `bounds`, the whole usable screen).
pub fn show(app: &mut EffectcraftApp, ui: &mut Ui, rect: Rect, bounds: Rect, u: f32) {
    let ctx = ui.ctx().clone();
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.header_bg);
    ui.painter().line_segment([rect.left_top(), rect.right_top()], egui::Stroke::new(1.0, t.separator));
    let Some(cid) = app.session.active_comp_id() else { return };
    let has_3d = app.session.active_comp().is_some_and(|c| c.has_3d());
    let ppp = ctx.pixels_per_point();
    let fit = viewer::last_fit(&ctx);
    let zoom = app.ui.viewer.zoom.unwrap_or(fit);
    let scale = app.viewer_scale(zoom, ppp);
    let mag = vt::magnification_label(zoom * ppp * 100.0);
    let res = vt::resolution_label(app.viewer_res(), scale);
    let vo = app.session.state.viewer.clone();
    let cur3 = app.session.state.views3d.get(&cid).map(|v| v.current).unwrap_or_default();
    let layout = match app.session.state.view_layout {
        2 => "2 Views",
        4 => "4 Views",
        _ => "1 View",
    };
    let v = &app.ui.viewer;
    let grid_any = v.grid || v.safe_margins || v.proportional_grid || v.rulers || v.guides;
    let has_roi = app.session.state.region_of_interest.is_some();
    let showing = vt::showing_snapshot(app, &ctx);
    let exp_label = if vo.exposure == 0.0 { "±0.0".to_string() } else { format!("{:+.1}", vo.exposure) };

    let iw = 46.0 * u;
    let items = vec![
        Item::text("mobile.view.mag", &mag, text_w(ui, &mag, u)),
        Item::text("mobile.view.res", &res, text_w(ui, &res, u)),
        Item::icon("mobile.view.transparency", Icon::Checker, iw).on(app.ui.viewer.transparency_grid),
        Item::icon("mobile.view.masks", Icon::MaskVis, iw).on(app.ui.viewer.show_masks),
        Item::icon("mobile.view.roi", Icon::Region, iw).on(has_roi || app.ui.viewer.roi_draw),
        Item::icon("mobile.view.grid", Icon::Grid, iw).on(grid_any),
        Item::icon("mobile.view.channel", Icon::Channel, iw).on(vo.channel != Channel::Rgb),
        Item::icon("mobile.view.resetExposure", Icon::Exposure, iw).on(vo.exposure != 0.0),
        Item::text("mobile.view.exposure", &exp_label, text_w(ui, &exp_label, u)),
        Item::icon("mobile.view.snapshot", Icon::Snapshot, iw),
        Item::icon("mobile.view.showSnapshot", Icon::ShowSnapshot, iw).on(showing).enabled(app.session.snapshot.is_some()),
        Item::icon("mobile.view.fast", Icon::Sparkle, iw).on(vo.fast_previews != FastPreviews::Off || app.ui.viewer.fast_preview),
        Item::text("mobile.view.3d", cur3.label(), text_w(ui, cur3.label(), u)).enabled(has_3d),
        Item::icon("mobile.view.extended", Icon::ExtendedViewer, iw).on(app.session.prefs.three_d.extended_viewer).enabled(has_3d),
        Item::text("mobile.view.layout", layout, text_w(ui, layout, u)),
    ];
    let out = kit::strip(app, ui, rect, "viewbar", &items, u);
    let mut open: Option<(Pop, Rect)> = ctx.data(|d| d.get_temp(key()));
    if let Some(i) = out.clicked {
        let anchor = out.rects[i];
        let id = items[i].id.as_str();
        let pop = match id {
            "mobile.view.mag" => Some(Pop::Mag),
            "mobile.view.res" => Some(Pop::Res),
            "mobile.view.grid" => Some(Pop::Grid),
            "mobile.view.channel" => Some(Pop::Channel),
            "mobile.view.exposure" => Some(Pop::Exposure),
            "mobile.view.fast" => Some(Pop::Fast),
            "mobile.view.3d" => Some(Pop::View3D),
            "mobile.view.layout" => Some(Pop::Layout),
            _ => None,
        };
        open = pop.map(|p| (p, anchor));
        match id {
            "mobile.view.transparency" => app.ui.viewer.transparency_grid = !app.ui.viewer.transparency_grid,
            "mobile.view.masks" => app.ui.viewer.show_masks = !app.ui.viewer.show_masks,
            "mobile.view.roi" => {
                if has_roi {
                    let _ = app.session.execute("view.setRegionOfInterest", json!({"rect": null}));
                    app.ui.viewer.roi_draw = false;
                } else {
                    app.ui.viewer.roi_draw = !app.ui.viewer.roi_draw;
                    if app.ui.viewer.roi_draw {
                        app.ui.status = "Drag on the preview to draw the region".into();
                    }
                }
            }
            "mobile.view.resetExposure" => {
                let _ = app.session.execute("view.resetExposure", json!({}));
            }
            "mobile.view.snapshot" => match app.session.execute("view.takeSnapshot", json!({"scale": scale.clamp(0.05, 1.0)})) {
                Ok(_) => app.ui.status = "Snapshot taken".into(),
                Err(e) => app.ui.status = e.to_string(),
            },
            "mobile.view.showSnapshot" => {
                let _ = app.session.execute("view.showSnapshot", json!({"value": !app.session.state.viewer.show_snapshot}));
            }
            "mobile.view.extended" => {
                let _ = app.session.execute("view.extendedViewer", json!({}));
            }
            "mobile.view.3d" if !has_3d => {}
            _ => {}
        }
        ctx.data_mut(|d| match open {
            Some(o) => {
                d.insert_temp(key(), o);
            }
            None => {
                d.remove::<(Pop, Rect)>(key());
            }
        });
    } else if let Some(i) = items.iter().position(|it| !it.enabled && it.id == "mobile.view.3d") {
        // A dimmed 3D control explains itself when tapped.
        let r = out.rects[i];
        if ctx.input(|i| i.pointer.any_click()) && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| r.contains(p)) && out.clicked.is_none() {
            app.ui.status = "Add a 3D layer to use the 3D views".into();
        }
    }

    let Some((pop, anchor)) = open else { return };
    let mut close;
    let w = 230.0 * u;
    match pop {
        Pop::Mag => {
            let cur = (zoom * ppp * 100.0 * 10.0).round() / 10.0;
            let rows: Vec<(String, bool)> = MAGS
                .iter()
                .map(|m| {
                    let on = match *m {
                        "Fit" => app.ui.viewer.zoom.is_none(),
                        s => s.trim_end_matches('%').parse::<f32>().is_ok_and(|v| app.ui.viewer.zoom.is_some() && (v - cur).abs() < 0.06),
                    };
                    (m.to_string(), on)
                })
                .collect();
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "mag", u));
            if let Some(i) = pick {
                app.ui.viewer.pan = [0.0, 0.0];
                app.ui.viewer.zoom = match MAGS[i] {
                    "Fit" => None,
                    "Fit up to 100%" => Some(fit.min(1.0 / ppp)),
                    s => s.trim_end_matches('%').parse::<f32>().ok().map(|v| v / 100.0 / ppp),
                };
                close = true;
            }
        }
        Pop::Res => {
            let r = app.viewer_res();
            let rows: Vec<(String, bool)> = vec![
                ("Full".into(), r == Resolution::Full),
                ("Half".into(), r == Resolution::Half),
                ("Third".into(), r == Resolution::Third),
                ("Quarter".into(), r == Resolution::Quarter),
                ("Auto".into(), r == Resolution::Auto),
            ];
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "res", u));
            if let Some(i) = pick {
                run(app, &ctx, ["view.res.full", "view.res.half", "view.res.third", "view.res.quarter", "view.res.auto"][i]);
                close = true;
            }
        }
        Pop::Grid => {
            let v = &app.ui.viewer;
            let rows: Vec<(String, bool)> = vec![
                ("Title/Action Safe".into(), v.safe_margins),
                ("Proportional Grid".into(), v.proportional_grid),
                ("Grid".into(), v.grid),
                ("Guides".into(), v.guides),
                ("Rulers".into(), v.rulers),
                ("3D Reference Axes".into(), app.session.prefs.three_d.show_reference_axes),
            ];
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "grid", u));
            if let Some(i) = pick {
                let v = &mut app.ui.viewer;
                match i {
                    0 => v.safe_margins = !v.safe_margins,
                    1 => v.proportional_grid = !v.proportional_grid,
                    2 => v.grid = !v.grid,
                    3 => v.guides = !v.guides,
                    4 => v.rulers = !v.rulers,
                    _ => app.session.prefs.three_d.show_reference_axes = !app.session.prefs.three_d.show_reference_axes,
                }
            }
        }
        Pop::Channel => {
            let mut rows: Vec<(String, bool)> = Channel::ALL.iter().map(|c| (c.label().to_string(), vo.channel == *c)).collect();
            rows.push(("Colorize".into(), vo.colorized));
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "channel", u));
            if let Some(i) = pick {
                let params = if i < Channel::ALL.len() { json!({"channel": Channel::ALL[i].id()}) } else { json!({"channel": vo.channel.id(), "colorized": !vo.colorized}) };
                let _ = app.session.execute("view.channel", params);
                if i < Channel::ALL.len() {
                    close = true;
                }
            }
        }
        Pop::Fast => {
            let rows: Vec<(String, bool)> = FastPreviews::ALL.iter().map(|m| (m.label().to_string(), vo.fast_previews == *m)).collect();
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "fast", u));
            if let Some(i) = pick {
                let _ = app.session.execute("view.fastPreviewMode", json!({"mode": FastPreviews::ALL[i].id()}));
                close = true;
            }
        }
        Pop::View3D => {
            let rows: Vec<(String, bool)> = View3D::ALL.iter().map(|v| (v.label().to_string(), *v == cur3)).collect();
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, (44.0 * u * rows.len() as f32 + 12.0 * u).min(bounds.height() * 0.6)), u, |ui, inner| pick = choices(app, ui, inner, &rows, "3d", u));
            if let Some(i) = pick {
                let _ = app.session.execute("view.set3DView", json!({"view": View3D::ALL[i].id()}));
                close = true;
            }
        }
        Pop::Layout => {
            let cur = app.session.state.view_layout;
            let rows: Vec<(String, bool)> = vec![("1 View".into(), cur <= 1), ("2 Views".into(), cur == 2), ("4 Views".into(), cur == 4)];
            let mut pick = None;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w, 44.0 * u * rows.len() as f32 + 12.0 * u), u, |ui, inner| pick = choices(app, ui, inner, &rows, "layout", u));
            if let Some(i) = pick {
                let views = [1, 2, 4][i];
                let _ = app.session.execute("view.layout", json!({ "views": views }));
                close = true;
            }
        }
        Pop::Exposure => {
            // A stepper: -  value  +, and Reset.
            let mut delta = 0.0f64;
            let mut reset = false;
            close = kit::popup(app.tokens, &ctx, "m-vb-pop", anchor, bounds, vec2(w + 20.0 * u, 128.0 * u), u, |ui, inner| {
                let t = app.tokens;
                let row = Rect::from_min_size(inner.min + vec2(0.0, 6.0 * u), vec2(inner.width(), 52.0 * u));
                let b = 52.0 * u;
                let minus = Rect::from_min_size(row.min, vec2(b, b));
                let plus = Rect::from_min_size(row.max - vec2(b, b), vec2(b, b));
                if kit::icon_tap(app, ui, minus, Icon::Minus, "Exposure down", false, true, "mobile.view.exposure.down", u) {
                    delta = -0.5;
                }
                if kit::icon_tap(app, ui, plus, Icon::Plus, "Exposure up", false, true, "mobile.view.exposure.up", u) {
                    delta = 0.5;
                }
                ui.painter().text(row.center(), egui::Align2::CENTER_CENTER, format!("{:+.1} stops", vo.exposure), crate::theme::Tokens::semibold(16.0), t.text);
                let rr = Rect::from_min_size(egui::pos2(inner.min.x, row.max.y + 8.0 * u), vec2(inner.width(), 46.0 * u));
                if kit::button(app, ui, rr, "Reset exposure", kit::Btn::Secondary, vo.exposure != 0.0, "mobile.view.exposure.reset", u) {
                    reset = true;
                }
            });
            if delta != 0.0 {
                let _ = app.session.execute("view.exposure", json!({"stops": (vo.exposure as f64 + delta).clamp(-40.0, 40.0)}));
            }
            if reset {
                let _ = app.session.execute("view.resetExposure", json!({}));
            }
        }
    }
    if close {
        ctx.data_mut(|d| d.remove::<(Pop, Rect)>(key()));
    }
}
