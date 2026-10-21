//! The phone's export page: pick what the render engine really supports (format, resolution,
//! quality, destination), start the render, watch its progress, stop it, and see where the file
//! was saved or why it failed. It drives the Render Queue (`renderQueue.add` / `.render` /
//! `.stop`) and reads its state, so a render started here also shows in the Render Queue panel.
//! Success is only reported from the queue item's own status.

use effectcraft_project::render_queue::{OutputFormat, RenderStatus};
use egui::{Align2, Color32, Id, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::icons::Icon;
use crate::mobile::data_dir;
use crate::mobile_kit::{self as kit, Btn};
use crate::theme::Tokens;
use crate::{EffectcraftApp, widgets};

#[derive(Clone)]
struct Export {
    format: OutputFormat,
    res: usize,
    best: bool,
    path: String,
}

const RES: [(&str, &str); 4] = [("Full", "full"), ("Half", "half"), ("Third", "third"), ("Quarter", "quarter")];

fn default_path(app: &EffectcraftApp, format: OutputFormat) -> String {
    let name = app
        .session
        .active_comp_id()
        .and_then(|c| app.session.project.item(c))
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "Composition".into())
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    let dir = format!("{}/Exports", data_dir());
    if format.is_sequence() { format!("{dir}/{name}_[#####].{}", format.extension()) } else { format!("{dir}/{name}.{}", format.extension()) }
}

fn swap_extension(path: &str, format: OutputFormat) -> String {
    let stem = format.output_stem(path);
    if format.is_sequence() && !stem.contains("[#") { format!("{stem}_[#####].{}", format.extension()) } else { format!("{stem}.{}", format.extension()) }
}

/// Draw the page over `full`. Returns true when it should close.
pub fn show(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, u: f32) -> bool {
    let t = app.tokens;
    let key = Id::new("m-export-state");
    let formats: Vec<OutputFormat> = app.session.exporter.as_ref().map(|e| e.formats()).unwrap_or_default();
    let mut ex: Export = ctx.data(|d| d.get_temp(key)).unwrap_or_else(|| {
        let f = formats.first().copied().unwrap_or_default();
        Export { format: f, res: 0, best: true, path: default_path(app, f) }
    });
    let rendering = app.session.is_rendering();
    let progress = app.session.render_progress();
    let latest = app.session.project.render_queue.last().cloned();
    let comp_info = app.session.active_comp().map(|c| (c.width, c.height, c.frame_rate.as_f64(), c.duration.seconds()));
    let comp_name = app.session.active_comp_id().and_then(|c| app.session.project.item(c)).map(|i| i.name.clone()).unwrap_or_default();
    if rendering {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    let mut close = false;
    let mut start = false;
    let mut stop = false;
    egui::Area::new(Id::new("m-export")).order(egui::Order::Foreground).fixed_pos(full.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(full);
        let (r, _) = ui.allocate_exact_size(full.size(), Sense::click());
        ui.painter().rect_filled(r, 0.0, t.panel_bg);
        let head = Rect::from_min_size(r.min, vec2(r.width(), 56.0 * u));
        ui.painter().rect_filled(head, 0.0, t.header_bg);
        ui.painter().line_segment([head.left_bottom(), head.right_bottom()], Stroke::new(1.0, t.app_bg));
        if kit::icon_tap(app, ui, Rect::from_min_size(head.min, vec2(52.0 * u, head.height())), Icon::ChevronLeft, "Back", false, true, "mobile.export.back", u) {
            close = true;
        }
        ui.painter().text(pos2(head.min.x + 52.0 * u, head.center().y), Align2::LEFT_CENTER, "Export", Tokens::semibold(17.0), t.text);

        // Footer: the one big action.
        let foot = Rect::from_min_max(pos2(r.min.x, r.max.y - 84.0 * u), r.max);
        ui.painter().rect_filled(foot, 0.0, t.header_bg);
        ui.painter().line_segment([foot.left_top(), foot.right_top()], Stroke::new(1.0, t.app_bg));
        let bw = (r.width() - 40.0 * u).min(440.0 * u);
        let br = Rect::from_center_size(foot.center(), vec2(bw, 52.0 * u));
        if rendering {
            if kit::button(app, ui, br, "Stop rendering", Btn::Danger, true, "mobile.export.stop", u) {
                stop = true;
            }
        } else if kit::button(app, ui, br, "Start render", Btn::Primary, !formats.is_empty() && comp_info.is_some(), "mobile.export.start", u) {
            start = true;
        }

        let body = Rect::from_min_max(pos2(r.min.x, head.max.y), pos2(r.max.x, foot.min.y));
        let cw = (body.width() - 32.0 * u).min(460.0 * u);
        let inner = Rect::from_min_max(pos2(body.center().x - cw / 2.0, body.min.y), pos2(body.center().x + cw / 2.0, body.max.y));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).id_salt("m-export-body"));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical().id_salt("m-export-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            ui.add_space(8.0 * u);
            // Composition summary.
            let card_h = 78.0 * u;
            let (cr, _) = ui.allocate_exact_size(vec2(cw, card_h), Sense::hover());
            ui.painter().rect_filled(cr, 14.0 * u, t.hover);
            widgets::text_fit(ui.painter(), pos2(cr.min.x + 16.0 * u, cr.min.y + 26.0 * u), Align2::LEFT_CENTER, &comp_name, Tokens::semibold(16.0), cw - 32.0 * u, t.text);
            let info = match comp_info {
                Some((w, h, fps, secs)) => format!("{w} × {h}   ·   {fps:.2} fps   ·   {secs:.1} s"),
                None => "No composition open".into(),
            };
            ui.painter().text(pos2(cr.min.x + 16.0 * u, cr.min.y + 52.0 * u), Align2::LEFT_CENTER, info, Tokens::mono(12.0), t.text_dim);

            // Format.
            kit::heading(app, ui, cw, "Format", u);
            if formats.is_empty() {
                let (er, _) = ui.allocate_exact_size(vec2(cw, 44.0 * u), Sense::hover());
                ui.painter().text(er.min + vec2(6.0 * u, 22.0 * u), Align2::LEFT_CENTER, "Export is not available in this build", Tokens::ui(13.0), t.text_dim);
            }
            let labels: Vec<String> = formats.iter().map(|f| f.label().to_string()).collect();
            let sel = formats.iter().position(|f| *f == ex.format);
            if let Some(i) = chips(app, ui, cw, &labels, |i| Some(i) == sel, "mobile.export.format", u) {
                ex.format = formats[i];
                ex.path = swap_extension(&ex.path, ex.format);
            }
            // Resolution and quality (movies and still sequences only).
            if !ex.format.is_audio_only() {
                kit::heading(app, ui, cw, "Resolution", u);
                let labels: Vec<String> = RES.iter().map(|(l, _)| l.to_string()).collect();
                let cur = ex.res;
                if let Some(i) = chips(app, ui, cw, &labels, |i| i == cur, "mobile.export.res", u) {
                    ex.res = i;
                }
                if let Some((w, h, ..)) = comp_info {
                    let k = [1.0, 0.5, 1.0 / 3.0, 0.25][ex.res];
                    let (rr, _) = ui.allocate_exact_size(vec2(cw, 24.0 * u), Sense::hover());
                    ui.painter().text(pos2(rr.min.x + 6.0 * u, rr.center().y), Align2::LEFT_CENTER, format!("Output {} × {}", (w as f32 * k).round(), (h as f32 * k).round()), Tokens::ui(12.0), t.text_faint);
                }
                kit::heading(app, ui, cw, "Quality", u);
                let labels = vec!["Best".to_string(), "Draft (faster)".to_string()];
                let best = ex.best;
                if let Some(i) = chips(app, ui, cw, &labels, |i| (i == 0) == best, "mobile.export.quality", u) {
                    ex.best = i == 0;
                }
            }
            // Destination.
            kit::heading(app, ui, cw, "Save to", u);
            let (fr, _) = ui.allocate_exact_size(vec2(cw, 46.0 * u), Sense::hover());
            app.auto.add("mobile.export.path", fr, "Save to");
            ui.painter().rect_filled(fr, 12.0 * u, t.field_bg);
            let resp = ui.put(fr.shrink2(vec2(12.0 * u, 0.0)), egui::TextEdit::singleline(&mut ex.path).font(Tokens::mono(12.0)).frame(egui::Frame::NONE).vertical_align(egui::Align::Center).margin(vec2(12.0 * u, 0.0)));
            if resp.has_focus() {
                ui.painter().rect_stroke(fr, 12.0 * u, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
            }
            let (nr, _) = ui.allocate_exact_size(vec2(cw, 40.0 * u), Sense::hover());
            ui.painter().text(pos2(nr.min.x + 6.0 * u, nr.min.y + 18.0 * u), Align2::LEFT_CENTER, "Files are saved inside the app's storage folder.", Tokens::ui(11.5), t.text_faint);

            // Progress and result.
            kit::heading(app, ui, cw, "Status", u);
            let (sr, _) = ui.allocate_exact_size(vec2(cw, 96.0 * u), Sense::hover());
            ui.painter().rect_filled(sr, 14.0 * u, t.hover);
            if rendering {
                let frac = progress.as_ref().filter(|p| p.total > 0).map(|p| (p.done as f32 / p.total as f32).clamp(0.0, 1.0)).unwrap_or(0.0);
                ui.painter().text(pos2(sr.min.x + 16.0 * u, sr.min.y + 26.0 * u), Align2::LEFT_CENTER, format!("Rendering… {:.0}%", frac * 100.0), Tokens::semibold(15.0), t.text);
                let bar = Rect::from_min_size(pos2(sr.min.x + 16.0 * u, sr.min.y + 50.0 * u), vec2(sr.width() - 32.0 * u, 10.0 * u));
                ui.painter().rect_filled(bar, 5.0 * u, t.field_bg);
                ui.painter().rect_filled(Rect::from_min_size(bar.min, vec2(bar.width() * frac, bar.height())), 5.0 * u, t.accent);
                if let Some(p) = &progress {
                    ui.painter().text(pos2(sr.min.x + 16.0 * u, sr.min.y + 76.0 * u), Align2::LEFT_CENTER, format!("Frame {} of {}", p.done, p.total), Tokens::mono(11.5), t.text_dim);
                }
            } else {
                let (title, detail, col) = match latest.as_ref().map(|i| (&i.status, i)) {
                    Some((RenderStatus::Done, it)) => ("Export finished", it.last_output.clone().unwrap_or_default(), Color32::from_rgb(0x4c, 0xc2, 0x7a)),
                    Some((RenderStatus::Failed(e), _)) => ("Export failed", e.clone(), Color32::from_rgb(0xe0, 0x5a, 0x5a)),
                    Some((RenderStatus::UserStopped, _)) => ("Export stopped", "The render was stopped.".to_string(), t.text_dim),
                    _ => ("Ready to export", "Choose the options above and tap Start render.".to_string(), t.text_dim),
                };
                ui.painter().text(pos2(sr.min.x + 16.0 * u, sr.min.y + 26.0 * u), Align2::LEFT_CENTER, title, Tokens::semibold(15.0), col);
                widgets::text_fit(ui.painter(), pos2(sr.min.x + 16.0 * u, sr.min.y + 56.0 * u), Align2::LEFT_CENTER, &detail, Tokens::mono(11.0), sr.width() - 32.0 * u, t.text_dim);
            }
            ui.add_space(24.0 * u);
        });
    });

    if stop {
        let _ = app.session.execute("renderQueue.stop", json!({}));
    }
    if start {
        let dir = std::path::Path::new(&ex.path).parent().map(|p| p.to_path_buf());
        if let Some(d) = dir.filter(|d| !d.as_os_str().is_empty())
            && let Err(e) = std::fs::create_dir_all(&d)
        {
            app.ui.status = format!("Cannot create {}: {e}", d.display());
        } else {
            let added = app.session.execute(
                "renderQueue.add",
                json!({"format": ex.format.label(), "output": ex.path, "quality": if ex.best { "best" } else { "draft" }, "resolution": RES[ex.res].1}),
            );
            match added.and_then(|_| app.session.execute("renderQueue.render", json!({"wait": false}))) {
                Ok(_) => {}
                Err(e) => app.ui.status = format!("Cannot start the render: {e}"),
            }
        }
    }
    if close {
        ctx.data_mut(|d| d.remove::<Export>(key));
    } else {
        ctx.data_mut(|d| d.insert_temp(key, ex));
    }
    close
}

/// Chips that wrap onto rows; returns the tapped one.
fn chips(app: &mut EffectcraftApp, ui: &mut egui::Ui, w: f32, labels: &[String], on: impl Fn(usize) -> bool, prefix: &str, u: f32) -> Option<usize> {
    let t = app.tokens;
    let mut hit = None;
    let mut rows: Vec<Vec<(usize, f32)>> = vec![vec![]];
    let mut x = 0.0;
    for (i, l) in labels.iter().enumerate() {
        let cw = (ui.painter().layout_no_wrap(l.clone(), Tokens::medium(13.0), Color32::WHITE).size().x + 28.0 * u).min(w);
        if x + cw > w && !rows.last().unwrap().is_empty() {
            rows.push(vec![]);
            x = 0.0;
        }
        rows.last_mut().unwrap().push((i, cw));
        x += cw + 8.0 * u;
    }
    for row in rows {
        let (area, _) = ui.allocate_exact_size(vec2(w, 48.0 * u), Sense::hover());
        let mut cx = area.min.x;
        for (i, cw) in row {
            let r = Rect::from_min_size(pos2(cx, area.min.y), vec2(cw, 40.0 * u));
            cx += cw + 8.0 * u;
            let resp = kit::tap(app, ui, r, &format!("{prefix}.{i}"), &labels[i]);
            let sel = on(i);
            ui.painter().rect_filled(r, r.height() / 2.0, if sel { t.accent } else if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
            ui.painter().text(r.center(), Align2::CENTER_CENTER, &labels[i], Tokens::medium(13.0), if sel { Color32::WHITE } else { t.text });
            if resp.clicked() {
                hit = Some(i);
            }
        }
    }
    hit
}
