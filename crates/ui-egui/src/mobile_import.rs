//! The phone's import screen: browse Camera / Pictures / Movies / Downloads / Music, tick videos,
//! photos and audio, and import them (optionally straight into the open composition).
//!
//! Phones have no desktop file dialog, so this is the picker. It lists folders through the
//! engine's Media Browser backend and imports with `mediaBrowser.import`, the same command the
//! desktop Media Browser uses. On Android the files are only readable after the user allows
//! Photos / Videos / Music; the host's `media_access` hook opens that system dialog.

use effectcraft_engine::media_browser as mb;
use egui::{Align2, Color32, Id, Pos2, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::icons::Icon;
use crate::theme::Tokens;
use crate::{EffectcraftApp, widgets};

#[derive(Clone, Default)]
struct ImportState {
    dir: Option<String>,
    sel: Vec<String>,
    /// 0 all, 1 video, 2 photos, 3 audio.
    filter: u8,
    asked: bool,
}

const FILTERS: [(&str, Option<&str>); 4] = [("All", None), ("Video", Some("video")), ("Photos", Some("image")), ("Audio", Some("audio"))];

fn key() -> Id {
    Id::new("mobile-import-state")
}

/// Draw the import screen over `full`. Returns true when it should close.
pub fn show(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, u: f32) -> bool {
    let t = app.tokens;
    let mut st: ImportState = ctx.data(|d| d.get_temp(key())).unwrap_or_default();
    let mut close = false;

    // Permission (Android only: the host provides the hook).
    let access = app.hooks.media_access.as_ref().map(|f| f(false));
    if access == Some(false)
        && !st.asked
        && let Some(f) = app.hooks.media_access.as_ref()
    {
        st.asked = true;
        f(true);
    }

    let Some(browser) = app.session.media_browser() else {
        app.ui.status = "Importing is not available here".to_string();
        return true;
    };
    let dir = st.dir.clone().unwrap_or_else(|| browser.home());
    let places = browser.places();
    // The listing is cached per folder for a second.
    let now = ctx.input(|i| i.time);
    let lid = Id::new(("mobile-import-list", dir.clone()));
    let cached: Option<(f64, Result<Vec<mb::Entry>, String>)> = ctx.data(|d| d.get_temp(lid));
    let listing = match cached {
        Some((at, l)) if now - at < 1.0 => l,
        _ => {
            let l = browser.list(&dir, true);
            ctx.data_mut(|d| d.insert_temp(lid, (now, l.clone())));
            l
        }
    };
    ctx.request_repaint_after(std::time::Duration::from_millis(500));

    let mut go: Option<String> = None;
    let mut toggle: Option<String> = None;
    let mut ask = false;
    let mut do_import: Option<bool> = None;
    let up = browser.parent(&dir);
    // Android: the system gallery and file manager, above the in-app folder browser.
    let has_pick = app.hooks.pick_media.is_some();
    let pick_h = if has_pick { 76.0 * u } else { 0.0 };
    let mut launch: Option<&'static str> = None;
    let mut pick_rects: Vec<(String, Rect)> = Vec::new();

    egui::Area::new(Id::new("m-import")).order(egui::Order::Foreground).fixed_pos(full.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(full);
        let (r, _) = ui.allocate_exact_size(full.size(), Sense::click());
        ui.painter().rect_filled(r, 0.0, t.panel_bg);
        let p = ui.painter().clone();

        // Header.
        let head = Rect::from_min_size(r.min, vec2(r.width(), 56.0 * u));
        p.rect_filled(head, 0.0, t.header_bg);
        let back = Rect::from_min_size(head.min, vec2(52.0 * u, head.height()));
        let resp = ui.interact(back, Id::new("m-imp-close"), Sense::click());
        crate::icons::paint(&p, Rect::from_center_size(back.center(), vec2(22.0 * u, 22.0 * u)), Icon::Close, t.icon_active);
        if resp.clicked() {
            close = true;
        }
        p.text(pos2(back.max.x, head.center().y - 7.0 * u), Align2::LEFT_CENTER, "Import", Tokens::semibold(16.0), t.text);
        p.text(pos2(back.max.x, head.center().y + 10.0 * u), Align2::LEFT_CENTER, "Videos, photos and audio", Tokens::ui(11.0), t.text_dim);

        // The system pickers: Photos & Videos open the gallery, Audio opens the file manager.
        if has_pick {
            let row = Rect::from_min_size(pos2(r.min.x, head.max.y), vec2(r.width(), pick_h));
            let bw = (row.width() - 36.0 * u) / 2.0;
            for (i, (title, sub, kind)) in [("Photos & Videos", "Open the gallery", "media"), ("Audio", "Open the file manager", "audio")].into_iter().enumerate() {
                let b = Rect::from_min_size(pos2(row.min.x + 12.0 * u + i as f32 * (bw + 12.0 * u), row.min.y + 10.0 * u), vec2(bw, pick_h - 14.0 * u));
                let rs = ui.interact(b, Id::new(("m-imp-pick", kind)), Sense::click());
                p.rect_filled(b, 14.0 * u, if rs.is_pointer_button_down_on() { t.accent_hover } else { t.accent });
                p.text(pos2(b.center().x, b.center().y - 8.0 * u), Align2::CENTER_CENTER, title, Tokens::semibold(14.0), Color32::WHITE);
                p.text(pos2(b.center().x, b.center().y + 10.0 * u), Align2::CENTER_CENTER, sub, Tokens::ui(11.0), Color32::from_white_alpha(200));
                app_auto_add(&mut pick_rects, format!("mobile.import.pick.{kind}"), b);
                if rs.clicked() {
                    launch = Some(kind);
                }
            }
        }

        // Places (Camera, Pictures, Movies…), scrolling sideways.
        let chips = Rect::from_min_size(pos2(r.min.x, head.max.y + pick_h), vec2(r.width(), 52.0 * u));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(chips).id_salt("m-imp-chips"));
        child.set_clip_rect(chips);
        egui::ScrollArea::horizontal().id_salt("m-imp-chips-scroll").auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(&mut child, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0 * u);
                for (name, path) in &places {
                    let w = p.layout_no_wrap(name.clone(), Tokens::medium(13.0), Color32::WHITE).size().x + 28.0 * u;
                    let (cr, _) = ui.allocate_exact_size(vec2(w, 36.0 * u), Sense::hover());
                    let cr = cr.translate(vec2(0.0, 8.0 * u));
                    let on = *path == dir;
                    let rs = ui.interact(cr, Id::new(("m-imp-place", path.as_str())), Sense::click());
                    let fill = if on { t.accent } else if rs.is_pointer_button_down_on() { t.pressed } else { t.hover };
                    ui.painter().rect_filled(cr, cr.height() / 2.0, fill);
                    ui.painter().text(cr.center(), Align2::CENTER_CENTER, name, Tokens::medium(13.0), if on { Color32::WHITE } else { t.text });
                    if rs.clicked() {
                        go = Some(path.clone());
                    }
                    ui.add_space(8.0 * u);
                }
            });
        });

        // Filter segments.
        let seg = Rect::from_min_size(pos2(r.min.x + 12.0 * u, chips.max.y + 2.0 * u), vec2(r.width() - 24.0 * u, 38.0 * u));
        p.rect_filled(seg, seg.height() / 2.0, t.field_bg);
        let sw = seg.width() / FILTERS.len() as f32;
        for (i, (label, _)) in FILTERS.iter().enumerate() {
            let cr = Rect::from_min_size(pos2(seg.min.x + sw * i as f32, seg.min.y), vec2(sw, seg.height()));
            let rs = ui.interact(cr, Id::new(("m-imp-filter", i)), Sense::click());
            let on = st.filter as usize == i;
            if on {
                p.rect_filled(cr.shrink(3.0 * u), (cr.height() - 6.0 * u) / 2.0, t.accent);
            }
            p.text(cr.center(), Align2::CENTER_CENTER, label, Tokens::medium(13.0), if on { Color32::WHITE } else { t.text_dim });
            if rs.clicked() {
                st.filter = i as u8;
            }
        }

        // Path row: up + the folder.
        let path_row = Rect::from_min_size(pos2(r.min.x, seg.max.y + 4.0 * u), vec2(r.width(), 44.0 * u));
        let upr = Rect::from_min_size(path_row.min, vec2(48.0 * u, path_row.height()));
        let ur = ui.interact(upr, Id::new("m-imp-up"), Sense::click());
        crate::icons::paint(&p, Rect::from_center_size(upr.center(), vec2(20.0 * u, 20.0 * u)), Icon::ChevronLeft, if up.is_some() { t.icon_active } else { t.text_faint });
        if ur.clicked()
            && let Some(parent) = &up
        {
            go = Some(parent.clone());
        }
        widgets::text_fit(&p, pos2(upr.max.x, path_row.center().y), Align2::LEFT_CENTER, &dir, Tokens::mono(11.0), r.width() - upr.width() - 12.0 * u, t.text_dim);

        // Bottom action bar.
        let bar = Rect::from_min_max(pos2(r.min.x, r.max.y - 80.0 * u), r.max);
        p.rect_filled(bar, 0.0, t.header_bg);
        p.line_segment([bar.left_top(), bar.right_top()], Stroke::new(1.0, t.app_bg));
        let n = st.sel.len();
        let can = n > 0;
        let label = if can { format!("{n} selected") } else { "Tap files to select".to_string() };
        p.text(pos2(bar.min.x + 14.0 * u, bar.center().y), Align2::LEFT_CENTER, label, Tokens::medium(12.0), if can { t.text } else { t.text_faint });
        let bh = 46.0 * u;
        let add_w = 118.0 * u;
        let imp_w = 96.0 * u;
        let add_r = Rect::from_min_size(pos2(bar.max.x - 12.0 * u - add_w, bar.center().y - bh / 2.0), vec2(add_w, bh));
        let imp_r = Rect::from_min_size(pos2(add_r.min.x - 8.0 * u - imp_w, add_r.min.y), vec2(imp_w, bh));
        for (rr, text, primary, id) in [(imp_r, "Import", false, "m-imp-go"), (add_r, "Add to comp", true, "m-imp-add")] {
            let rs = ui.interact(rr, Id::new(id), Sense::click());
            let fill = match (can, primary, rs.is_pointer_button_down_on()) {
                (false, _, _) => t.field_bg,
                (true, true, true) => t.accent_hover,
                (true, true, false) => t.accent,
                (true, false, true) => t.pressed,
                (true, false, false) => t.hover,
            };
            p.rect_filled(rr, bh / 2.0, fill);
            p.text(rr.center(), Align2::CENTER_CENTER, text, Tokens::semibold(13.0), if can { Color32::WHITE } else { t.text_faint });
            if can && rs.clicked() {
                do_import = Some(primary);
            }
        }

        // The list.
        let list = Rect::from_min_max(pos2(r.min.x, path_row.max.y), pos2(r.max.x, bar.min.y));
        if access == Some(false) {
            let c = list.center();
            p.text(pos2(c.x, c.y - 60.0 * u), Align2::CENTER_CENTER, "Allow access to your media", Tokens::semibold(16.0), t.text);
            p.text(pos2(c.x, c.y - 30.0 * u), Align2::CENTER_CENTER, "Photos, videos and music are only readable", Tokens::ui(12.0), t.text_dim);
            p.text(pos2(c.x, c.y - 12.0 * u), Align2::CENTER_CENTER, "after you allow it in the Android dialog.", Tokens::ui(12.0), t.text_dim);
            let b = Rect::from_center_size(pos2(c.x, c.y + 40.0 * u), vec2(200.0 * u, 48.0 * u));
            let rs = ui.interact(b, Id::new("m-imp-allow"), Sense::click());
            p.rect_filled(b, b.height() / 2.0, if rs.is_pointer_button_down_on() { t.accent_hover } else { t.accent });
            p.text(b.center(), Align2::CENTER_CENTER, "Allow access", Tokens::semibold(14.0), Color32::WHITE);
            p.text(pos2(c.x, c.y + 86.0 * u), Align2::CENTER_CENTER, "No dialog? Settings > Apps > EffectCraft > Permissions", Tokens::ui(11.0), t.text_faint);
            if rs.clicked() {
                ask = true;
            }
            return;
        }
        let entries = match &listing {
            Ok(e) => e.clone(),
            Err(msg) => {
                p.text(list.center(), Align2::CENTER_CENTER, msg, Tokens::ui(12.0), t.text_dim);
                return;
            }
        };
        let want = FILTERS[st.filter as usize].1;
        let shown: Vec<&mb::Entry> = entries.iter().filter(|e| e.is_dir || want.is_none_or(|w| e.kind == Some(w))).collect();
        if shown.is_empty() {
            p.text(list.center(), Align2::CENTER_CENTER, "Nothing to import in this folder", Tokens::ui(13.0), t.text_dim);
            return;
        }
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(list).id_salt("m-imp-list"));
        child.set_clip_rect(list);
        let rh = 60.0 * u;
        egui::ScrollArea::vertical().id_salt("m-imp-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            for e in shown {
                let (row, _) = ui.allocate_exact_size(vec2(list.width(), rh), Sense::hover());
                let rs = ui.interact(row, Id::new(("m-imp-row", e.path.as_str())), Sense::click());
                let picked = st.sel.contains(&e.path);
                if picked {
                    ui.painter().rect_filled(row, 0.0, t.accent.gamma_multiply(0.22));
                } else if rs.is_pointer_button_down_on() {
                    ui.painter().rect_filled(row, 0.0, t.pressed);
                }
                let icon = if e.is_dir {
                    Icon::Folder
                } else {
                    match e.kind {
                        Some("video") => Icon::Footage,
                        Some("audio") => Icon::Audio,
                        _ => Icon::Image,
                    }
                };
                crate::icons::paint(ui.painter(), Rect::from_center_size(pos2(row.min.x + 30.0 * u, row.center().y), vec2(26.0 * u, 26.0 * u)), icon, if e.is_dir { t.text_dim } else { t.icon_active });
                widgets::text_fit(ui.painter(), pos2(row.min.x + 60.0 * u, row.center().y - 8.0 * u), Align2::LEFT_CENTER, &e.name, Tokens::medium(14.0), row.width() - 130.0 * u, t.text);
                let meta = if e.is_dir { "Folder".to_string() } else { format!("{}  ·  {}", e.kind.unwrap_or("file"), human_size(e.size)) };
                ui.painter().text(pos2(row.min.x + 60.0 * u, row.center().y + 11.0 * u), Align2::LEFT_CENTER, meta, Tokens::ui(11.0), t.text_faint);
                if e.is_dir {
                    crate::icons::paint(ui.painter(), Rect::from_center_size(pos2(row.max.x - 26.0 * u, row.center().y), vec2(18.0 * u, 18.0 * u)), Icon::ChevronRight, t.text_faint);
                } else {
                    let c = pos2(row.max.x - 30.0 * u, row.center().y);
                    ui.painter().circle_filled(c, 13.0 * u, if picked { t.accent } else { Color32::TRANSPARENT });
                    ui.painter().circle_stroke(c, 13.0 * u, Stroke::new(2.0, if picked { t.accent } else { t.text_faint }));
                    if picked {
                        let s = Stroke::new(2.4, Color32::WHITE);
                        ui.painter().line_segment([c + vec2(-5.0 * u, 0.0), c + vec2(-1.5 * u, 4.0 * u)], s);
                        ui.painter().line_segment([c + vec2(-1.5 * u, 4.0 * u), c + vec2(6.0 * u, -4.5 * u)], s);
                    }
                }
                ui.painter().line_segment([Pos2::new(row.min.x + 60.0 * u, row.max.y), row.right_bottom()], Stroke::new(1.0, t.app_bg));
                if rs.clicked() {
                    if e.is_dir {
                        go = Some(e.path.clone());
                    } else {
                        toggle = Some(e.path.clone());
                    }
                }
            }
        });
    });

    for (id, rect) in pick_rects {
        app.auto.add(&id, rect, "Import picker");
    }
    if let Some(kind) = launch
        && let Some(open) = app.hooks.pick_media.as_ref()
    {
        if open(kind) {
            // The result is polled by the editor shell (`mobile::show`) once the picker closes.
            ctx.data_mut(|d| d.insert_temp(Id::new("mobile-pick-pending"), true));
            close = true;
        } else {
            app.ui.status = "Could not open the picker".to_string();
        }
    }
    if ask && let Some(f) = app.hooks.media_access.as_ref() {
        f(true);
    }
    if let Some(d) = go {
        st.dir = Some(d);
    }
    if let Some(path) = toggle {
        match st.sel.iter().position(|p| *p == path) {
            Some(i) => {
                st.sel.remove(i);
            }
            None => st.sel.push(path),
        }
    }
    if let Some(add_to_comp) = do_import {
        let add = add_to_comp && app.session.active_comp_id().is_some();
        match app.session.execute("mediaBrowser.import", json!({"paths": st.sel, "addToComp": add})) {
            Ok(v) => {
                let n = v.get("items").and_then(|i| i.as_array()).map_or(st.sel.len(), |i| i.len());
                app.ui.status = if add { format!("Added {n} item(s) to the composition") } else { format!("Imported {n} item(s) to Project") };
                st.sel.clear();
                close = true;
            }
            Err(e) => app.ui.status = e.to_string(),
        }
    }
    if close {
        st = ImportState { asked: st.asked, ..ImportState::default() };
    }
    ctx.data_mut(|d| d.insert_temp(key(), st));
    close
}

fn app_auto_add(list: &mut Vec<(String, Rect)>, id: String, rect: Rect) {
    list.push((id, rect));
}

fn human_size(b: u64) -> String {
    const K: f64 = 1024.0;
    let b = b as f64;
    if b < K {
        format!("{b:.0} B")
    } else if b < K * K {
        format!("{:.0} KB", b / K)
    } else if b < K * K * K {
        format!("{:.1} MB", b / (K * K))
    } else {
        format!("{:.2} GB", b / (K * K * K))
    }
}
