//! Phone Home / Project Management: a welcome line, New Project, Open Project, the recent
//! projects and one small community box. New Project opens the composition settings as a
//! full-screen page (it scrolls inside; OK and Cancel stay pinned), and OK opens the editor.
//! Also the unsaved-changes prompt used when leaving the editor, and the project save helper.

use egui::{Align2, Color32, Id, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::icons::Icon;
use crate::mobile_kit::{self as kit, Btn};
use crate::mobile::data_dir;
use crate::theme::Tokens;
use crate::{EffectcraftApp, widgets};

/// Community and social links shown on Home: label, address. Edit this list to change them.
const COMMUNITY: [(&str, &str); 3] =
    [("Discord", "https://discord.gg/artcraft"), ("GitHub", "https://github.com/storytold/effectcraft"), ("Report a bug", "https://github.com/storytold/effectcraft/issues")];

#[derive(Clone, Copy, PartialEq, Default)]
enum Screen {
    #[default]
    Main,
    New,
    Open,
}

fn screen_key() -> Id {
    Id::new("m-home-screen")
}

/// Which Home screen to show next: 0 main, 1 New Project, 2 Open Project.
pub fn goto(ctx: &egui::Context, target: u8) {
    let s = match target {
        1 => Screen::New,
        2 => Screen::Open,
        _ => Screen::Main,
    };
    ctx.data_mut(|d| d.insert_temp(screen_key(), s));
}

/// Where new projects are saved.
pub fn projects_dir() -> String {
    format!("{}/Projects", data_dir())
}

/// Save the open project: in place when it has a file, else into the projects folder under the
/// composition's name. Returns the saved path.
pub fn save_project(app: &mut EffectcraftApp, force_new: bool) -> Result<String, String> {
    if let Some(p) = app.session.path.clone().filter(|_| !force_new) {
        app.session.execute("file.save", json!({})).map_err(|e| e.to_string())?;
        return Ok(p);
    }
    let dir = projects_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create {dir}: {e}"))?;
    let base = app
        .session
        .active_comp_id()
        .and_then(|c| app.session.project.item(c))
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "Untitled".into())
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    let mut path = format!("{dir}/{base}.ecproj");
    let mut n = 2;
    while std::path::Path::new(&path).exists() {
        path = format!("{dir}/{base}-{n}.ecproj");
        n += 1;
    }
    app.session.execute("file.saveAs", json!({"path": path})).map_err(|e| e.to_string())?;
    Ok(path)
}

/// The Home page.
pub fn show(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, full: Rect, u: f32) {
    let t = app.tokens;
    let screen: Screen = ctx.data(|d| d.get_temp(screen_key())).unwrap_or_default();
    ui.painter().rect_filled(full, 0.0, t.app_bg);
    match screen {
        Screen::Main => main_screen(app, ui, ctx, full, u),
        Screen::New => {
            if let Some(created) = new_comp_form(app, ctx, full, u, true) {
                ctx.data_mut(|d| d.insert_temp(screen_key(), Screen::Main));
                if created {
                    app.ui.start_screen = false;
                }
            }
        }
        Screen::Open => {
            if open_picker(app, ctx, full, u) {
                ctx.data_mut(|d| d.insert_temp(screen_key(), Screen::Main));
            }
        }
    }
}

fn main_screen(app: &mut EffectcraftApp, ui: &mut Ui, ctx: &egui::Context, full: Rect, u: f32) {
    let t = app.tokens;
    let w = (full.width() - 40.0 * u).min(440.0 * u);
    let x = full.center().x - w / 2.0;
    let mut y = full.min.y + (full.height() * 0.07).max(24.0 * u);

    // Welcome.
    ui.painter().text(pos2(full.center().x, y + 16.0 * u), Align2::CENTER_CENTER, "Welcome to", Tokens::ui(15.0), t.text_dim);
    ui.painter().text(pos2(full.center().x, y + 44.0 * u), Align2::CENTER_CENTER, "EffectCraft", Tokens::semibold(30.0), t.text);
    y += 92.0 * u;

    // New Project, Open Project: centred, one under the other.
    let bh = 56.0 * u;
    if kit::button(app, ui, Rect::from_min_size(pos2(x, y), vec2(w, bh)), "New Project", Btn::Primary, true, "mobile.home.new", u) {
        ctx.data_mut(|d| d.insert_temp(screen_key(), Screen::New));
    }
    y += bh + 14.0 * u;
    if kit::button(app, ui, Rect::from_min_size(pos2(x, y), vec2(w, bh)), "Open Project", Btn::Secondary, true, "mobile.home.open", u) {
        ctx.data_mut(|d| d.insert_temp(screen_key(), Screen::Open));
    }
    y += bh + 26.0 * u;

    // Community box, pinned to the bottom.
    let box_h = 96.0 * u;
    let cbox = Rect::from_min_size(pos2(x, full.max.y - box_h - 20.0 * u), vec2(w, box_h));
    ui.painter().rect_filled(cbox, 16.0 * u, t.panel_bg);
    ui.painter().rect_stroke(cbox, 16.0 * u, Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    ui.painter().text(pos2(cbox.min.x + 16.0 * u, cbox.min.y + 18.0 * u), Align2::LEFT_CENTER, "Community & Social Links", Tokens::semibold(12.5), t.text_dim);
    let gap = 8.0 * u;
    let cw = (cbox.width() - 24.0 * u - gap * (COMMUNITY.len() as f32 - 1.0)) / COMMUNITY.len() as f32;
    for (i, (label, url)) in COMMUNITY.iter().enumerate() {
        let r = Rect::from_min_size(pos2(cbox.min.x + 12.0 * u + (cw + gap) * i as f32, cbox.min.y + 38.0 * u), vec2(cw, 40.0 * u));
        if kit::button(app, ui, r, label, Btn::Secondary, true, &format!("mobile.home.link.{label}"), u) {
            ctx.open_url(egui::OpenUrl::new_tab(*url));
        }
    }

    // Recent projects fill the space between.
    let list = Rect::from_min_max(pos2(x, y), pos2(x + w, cbox.min.y - 16.0 * u));
    ui.painter().text(pos2(list.min.x + 4.0 * u, list.min.y + 10.0 * u), Align2::LEFT_CENTER, "Recent projects", Tokens::semibold(13.0), t.text_dim);
    let body = Rect::from_min_max(pos2(list.min.x, list.min.y + 26.0 * u), list.max);
    let recents: Vec<String> = app.session.prefs.recent_projects.clone();
    if recents.is_empty() {
        ui.painter().text(body.center(), Align2::CENTER_CENTER, "No recent projects yet", Tokens::ui(13.0), t.text_faint);
        return;
    }
    let mut open: Option<usize> = None;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body).id_salt("m-home-recents"));
    child.set_clip_rect(body);
    egui::ScrollArea::vertical().id_salt("m-home-recents-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
        for (i, path) in recents.iter().enumerate() {
            let name = std::path::Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or(path).to_string();
            let (r, _) = ui.allocate_exact_size(vec2(body.width(), 60.0 * u), Sense::hover());
            let resp = kit::tap(app, ui, r, &format!("mobile.home.recent.{i}"), &name);
            if resp.is_pointer_button_down_on() {
                ui.painter().rect_filled(r, 12.0 * u, t.pressed);
            }
            crate::icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 24.0 * u, r.center().y), vec2(24.0 * u, 24.0 * u)), Icon::Comp, t.icon_active);
            widgets::text_fit(ui.painter(), pos2(r.min.x + 54.0 * u, r.center().y - 9.0 * u), Align2::LEFT_CENTER, &name, Tokens::semibold(14.0), r.width() - 66.0 * u, t.text);
            widgets::text_fit(ui.painter(), pos2(r.min.x + 54.0 * u, r.center().y + 11.0 * u), Align2::LEFT_CENTER, path, Tokens::ui(10.5), r.width() - 66.0 * u, t.text_faint);
            ui.painter().line_segment([pos2(r.min.x + 54.0 * u, r.max.y), r.right_bottom()], Stroke::new(1.0, t.separator));
            if resp.clicked() {
                open = Some(i);
            }
        }
    });
    if let Some(i) = open {
        match app.session.execute("file.openRecent", json!({"index": i})) {
            Ok(_) => app.ui.start_screen = false,
            Err(e) => app.ui.status = format!("Cannot open the project: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Open Project
// ---------------------------------------------------------------------------------------------

/// Pick a project file from the projects folder or any folder; returns true when done.
fn open_picker(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, u: f32) -> bool {
    let t = app.tokens;
    let Some(browser) = app.session.media_browser() else {
        app.ui.status = "Browsing files is not available here".into();
        return true;
    };
    let did = Id::new("m-open-dir");
    let default = if std::path::Path::new(&projects_dir()).is_dir() { projects_dir() } else { browser.home() };
    let dir: String = ctx.data(|d| d.get_temp(did)).unwrap_or(default);
    let listing = browser.list(&dir, false);
    let up = browser.parent(&dir);
    let mut close = false;
    let mut go: Option<String> = None;
    let mut open: Option<String> = None;
    egui::Area::new(Id::new("m-open")).order(egui::Order::Foreground).fixed_pos(full.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(full);
        let (r, _) = ui.allocate_exact_size(full.size(), Sense::click());
        ui.painter().rect_filled(r, 0.0, t.panel_bg);
        let head = Rect::from_min_size(r.min, vec2(r.width(), 56.0 * u));
        ui.painter().rect_filled(head, 0.0, t.header_bg);
        if kit::icon_tap(app, ui, Rect::from_min_size(head.min, vec2(52.0 * u, head.height())), Icon::ChevronLeft, "Back", false, true, "mobile.open.back", u) {
            close = true;
        }
        ui.painter().text(pos2(head.min.x + 52.0 * u, head.center().y), Align2::LEFT_CENTER, "Open Project", Tokens::semibold(17.0), t.text);
        // Places, then the path row.
        let places = browser.places();
        let mut px = r.min.x + 12.0 * u;
        let py = head.max.y + 8.0 * u;
        let mut chips: Vec<(String, String)> = vec![("Projects".into(), projects_dir())];
        chips.extend(places);
        for (name, path) in &chips {
            let cw = ui.painter().layout_no_wrap(name.clone(), Tokens::medium(13.0), Color32::WHITE).size().x + 28.0 * u;
            if px + cw > r.max.x - 8.0 * u {
                break;
            }
            let cr = Rect::from_min_size(pos2(px, py), vec2(cw, 36.0 * u));
            px += cw + 8.0 * u;
            let on = *path == dir;
            let resp = kit::tap(app, ui, cr, &format!("mobile.open.place.{name}"), name);
            ui.painter().rect_filled(cr, cr.height() / 2.0, if on { t.accent } else if resp.is_pointer_button_down_on() { t.pressed } else { t.hover });
            ui.painter().text(cr.center(), Align2::CENTER_CENTER, name, Tokens::medium(13.0), if on { Color32::WHITE } else { t.text });
            if resp.clicked() {
                go = Some(path.clone());
            }
        }
        let path_row = Rect::from_min_size(pos2(r.min.x, py + 44.0 * u), vec2(r.width(), 44.0 * u));
        let upr = Rect::from_min_size(path_row.min, vec2(48.0 * u, path_row.height()));
        if kit::icon_tap(app, ui, upr, Icon::ChevronLeft, "Up", false, up.is_some(), "mobile.open.up", u)
            && let Some(p) = &up
        {
            go = Some(p.clone());
        }
        widgets::text_fit(ui.painter(), pos2(upr.max.x, path_row.center().y), Align2::LEFT_CENTER, &dir, Tokens::mono(11.0), r.width() - upr.width() - 12.0 * u, t.text_dim);
        // Folders and project files.
        let body = Rect::from_min_max(pos2(r.min.x, path_row.max.y), r.max);
        let entries: Vec<_> = match &listing {
            Ok(e) => e.iter().filter(|e| e.is_dir || e.name.ends_with(".ecproj")).cloned().collect(),
            Err(msg) => {
                ui.painter().text(body.center(), Align2::CENTER_CENTER, msg, Tokens::ui(12.0), t.text_dim);
                return;
            }
        };
        if entries.is_empty() {
            ui.painter().text(body.center(), Align2::CENTER_CENTER, "No projects in this folder", Tokens::ui(13.0), t.text_faint);
            return;
        }
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body).id_salt("m-open-list"));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical().id_salt("m-open-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            for (i, e) in entries.iter().enumerate() {
                let icon = if e.is_dir { Icon::Folder } else { Icon::Comp };
                if kit::list_row(app, ui, body.width(), 56.0 * u, Some(icon), &e.name, if e.is_dir { Some("›") } else { None }, None, true, &format!("mobile.open.entry.{i}"), u) {
                    if e.is_dir {
                        go = Some(e.path.clone());
                    } else {
                        open = Some(e.path.clone());
                    }
                }
            }
        });
    });
    if let Some(d) = go {
        ctx.data_mut(|data| data.insert_temp(did, d));
    }
    if let Some(p) = open {
        match app.session.execute("file.open", json!({"path": p})) {
            Ok(_) => {
                app.ui.start_screen = false;
                close = true;
            }
            Err(e) => app.ui.status = format!("Cannot open the project: {e}"),
        }
    }
    if close {
        ctx.data_mut(|d| d.remove::<String>(did));
    }
    close
}

// ---------------------------------------------------------------------------------------------
// Composition settings (New Project / New Composition)
// ---------------------------------------------------------------------------------------------

/// Preset: label, width, height.
const PRESETS: [(&str, u32, u32); 6] = [("Shorts / Reels 9:16", 1080, 1920), ("Square 1:1", 1080, 1080), ("Portrait 4:5", 1080, 1350), ("HD 16:9", 1920, 1080), ("HD 720p", 1280, 720), ("4K UHD", 3840, 2160)];
const RATES: [f64; 5] = [24.0, 25.0, 30.0, 50.0, 60.0];
const BACKGROUNDS: [(&str, [f32; 3]); 4] = [("Black", [0.0, 0.0, 0.0]), ("White", [1.0, 1.0, 1.0]), ("Dark gray", [0.12, 0.12, 0.13]), ("Midnight", [0.03, 0.04, 0.12])];

#[derive(Clone)]
struct Form {
    name: String,
    width: String,
    height: String,
    rate: String,
    secs: String,
    bg: usize,
}

impl Default for Form {
    fn default() -> Self {
        Form { name: "Comp 1".into(), width: "1080".into(), height: "1920".into(), rate: "30".into(), secs: "15".into(), bg: 0 }
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn field(app: &mut EffectcraftApp, ui: &mut Ui, w: f32, label: &str, text: &mut String, numeric: bool, id: &str, u: f32) {
    let t = app.tokens;
    let (cap, _) = ui.allocate_exact_size(vec2(w, 26.0 * u), Sense::hover());
    ui.painter().text(pos2(cap.min.x + 4.0 * u, cap.max.y - 6.0 * u), Align2::LEFT_CENTER, label, Tokens::semibold(12.0), t.text_dim);
    let (r, _) = ui.allocate_exact_size(vec2(w, 46.0 * u), Sense::hover());
    app.auto.add(id, r, label);
    ui.painter().rect_filled(r, 12.0 * u, t.field_bg);
    let mut edit = egui::TextEdit::singleline(text).font(Tokens::medium(15.0)).frame(egui::Frame::NONE).vertical_align(egui::Align::Center).margin(vec2(14.0 * u, 0.0));
    if numeric {
        edit = edit.hint_text("0");
    }
    let resp = ui.put(r.shrink2(vec2(14.0 * u, 0.0)), edit);
    if resp.has_focus() {
        ui.painter().rect_stroke(r, 12.0 * u, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
    }
}

/// Chips in rows that wrap; returns the tapped one.
fn chips(app: &mut EffectcraftApp, ui: &mut Ui, w: f32, labels: &[String], on: impl Fn(usize) -> bool, prefix: &str, u: f32) -> Option<usize> {
    let t = app.tokens;
    let mut hit = None;
    let mut x = 0.0;
    let mut rows: Vec<Vec<(usize, f32)>> = vec![vec![]];
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
        let (area, _) = ui.allocate_exact_size(vec2(w, 40.0 * u + 8.0 * u), Sense::hover());
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

/// The composition settings page. `project`: also start a new project first. Returns
/// `Some(true)` when the composition was created, `Some(false)` on cancel, `None` while open.
pub fn new_comp_form(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, u: f32, project: bool) -> Option<bool> {
    let t = app.tokens;
    let fid = Id::new("m-comp-form");
    let mut f: Form = ctx.data(|d| d.get_temp(fid)).unwrap_or_default();
    let mut result: Option<bool> = None;
    let mut create = false;

    egui::Area::new(Id::new("m-comp-page")).order(egui::Order::Foreground).fixed_pos(full.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(full);
        let (r, _) = ui.allocate_exact_size(full.size(), Sense::click());
        ui.painter().rect_filled(r, 0.0, t.panel_bg);
        let head = Rect::from_min_size(r.min, vec2(r.width(), 56.0 * u));
        ui.painter().rect_filled(head, 0.0, t.header_bg);
        ui.painter().line_segment([head.left_bottom(), head.right_bottom()], Stroke::new(1.0, t.app_bg));
        ui.painter().text(pos2(r.center().x, head.center().y), Align2::CENTER_CENTER, if project { "New Project" } else { "New Composition" }, Tokens::semibold(17.0), t.text);
        if kit::icon_tap(app, ui, Rect::from_min_size(head.min, vec2(52.0 * u, head.height())), Icon::Close, "Cancel", false, true, "mobile.comp.close", u) {
            result = Some(false);
        }
        // Footer: Cancel and OK, side by side and centred.
        let foot = Rect::from_min_max(pos2(r.min.x, r.max.y - 84.0 * u), r.max);
        ui.painter().rect_filled(foot, 0.0, t.header_bg);
        ui.painter().line_segment([foot.left_top(), foot.right_top()], Stroke::new(1.0, t.app_bg));
        let cw = (r.width() - 24.0 * u - 12.0 * u).min(2.0 * 200.0 * u + 12.0 * u) / 2.0;
        let fx = r.center().x - (cw * 2.0 + 12.0 * u) / 2.0;
        let by = foot.center().y - 24.0 * u;
        if kit::button(app, ui, Rect::from_min_size(pos2(fx, by), vec2(cw, 48.0 * u)), "Cancel", Btn::Secondary, true, "mobile.comp.cancel", u) {
            result = Some(false);
        }
        let (w, h, fps, secs) = (f.width.trim().parse::<u32>().ok(), f.height.trim().parse::<u32>().ok(), f.rate.trim().parse::<f64>().ok(), f.secs.trim().parse::<f64>().ok());
        let valid = w.is_some_and(|v| (4..=30000).contains(&v)) && h.is_some_and(|v| (4..=30000).contains(&v)) && fps.is_some_and(|v| (1.0..=240.0).contains(&v)) && secs.is_some_and(|v| v >= 0.1) && !f.name.trim().is_empty();
        if kit::button(app, ui, Rect::from_min_size(pos2(fx + cw + 12.0 * u, by), vec2(cw, 48.0 * u)), "OK", Btn::Primary, valid, "mobile.comp.ok", u) {
            create = true;
        }
        // The settings, scrolling inside the page.
        let body = Rect::from_min_max(pos2(r.min.x, head.max.y), pos2(r.max.x, foot.min.y));
        let cwid = (body.width() - 32.0 * u).min(460.0 * u);
        let cx = body.center().x - cwid / 2.0;
        let inner = Rect::from_min_max(pos2(cx, body.min.y), pos2(cx + cwid, body.max.y));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).id_salt("m-comp-body"));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical().id_salt("m-comp-scroll").auto_shrink([false, false]).show(&mut child, |ui| {
            ui.add_space(8.0 * u);
            field(app, ui, cwid, "Composition name", &mut f.name, false, "mobile.comp.name", u);
            // Presets.
            let (cap, _) = ui.allocate_exact_size(vec2(cwid, 34.0 * u), Sense::hover());
            ui.painter().text(pos2(cap.min.x + 4.0 * u, cap.max.y - 6.0 * u), Align2::LEFT_CENTER, "Preset", Tokens::semibold(12.0), t.text_dim);
            let labels: Vec<String> = PRESETS.iter().map(|(l, ..)| l.to_string()).collect();
            let cur = (f.width.parse::<u32>().unwrap_or(0), f.height.parse::<u32>().unwrap_or(0));
            if let Some(i) = chips(app, ui, cwid, &labels, |i| (PRESETS[i].1, PRESETS[i].2) == cur, "mobile.comp.preset", u) {
                f.width = PRESETS[i].1.to_string();
                f.height = PRESETS[i].2.to_string();
            }
            // Size.
            let half = (cwid - 12.0 * u) / 2.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0 * u;
                ui.vertical(|ui| field(app, ui, half, "Width (px)", &mut f.width, true, "mobile.comp.width", u));
                ui.vertical(|ui| field(app, ui, half, "Height (px)", &mut f.height, true, "mobile.comp.height", u));
            });
            let ratio = match (f.width.parse::<u32>(), f.height.parse::<u32>()) {
                (Ok(a), Ok(b)) if a > 0 && b > 0 => {
                    let g = gcd(a, b);
                    format!("Aspect ratio {}:{}", a / g, b / g)
                }
                _ => "Aspect ratio -".to_string(),
            };
            let (ar, _) = ui.allocate_exact_size(vec2(cwid, 30.0 * u), Sense::hover());
            ui.painter().text(pos2(ar.min.x + 4.0 * u, ar.center().y), Align2::LEFT_CENTER, ratio, Tokens::ui(12.0), t.text_faint);
            // Frame rate.
            let (cap, _) = ui.allocate_exact_size(vec2(cwid, 30.0 * u), Sense::hover());
            ui.painter().text(pos2(cap.min.x + 4.0 * u, cap.max.y - 6.0 * u), Align2::LEFT_CENTER, "Frame rate (fps)", Tokens::semibold(12.0), t.text_dim);
            let rl: Vec<String> = RATES.iter().map(|r| format!("{r}")).collect();
            let cur = f.rate.parse::<f64>().unwrap_or(0.0);
            if let Some(i) = chips(app, ui, cwid, &rl, |i| (RATES[i] - cur).abs() < 1e-6, "mobile.comp.rate", u) {
                f.rate = format!("{}", RATES[i]);
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0 * u;
                ui.vertical(|ui| field(app, ui, half, "Custom frame rate", &mut f.rate, true, "mobile.comp.fps", u));
                ui.vertical(|ui| field(app, ui, half, "Duration (seconds)", &mut f.secs, true, "mobile.comp.duration", u));
            });
            // Background.
            let (cap, _) = ui.allocate_exact_size(vec2(cwid, 34.0 * u), Sense::hover());
            ui.painter().text(pos2(cap.min.x + 4.0 * u, cap.max.y - 6.0 * u), Align2::LEFT_CENTER, "Background colour", Tokens::semibold(12.0), t.text_dim);
            let bl: Vec<String> = BACKGROUNDS.iter().map(|(l, _)| l.to_string()).collect();
            let bg = f.bg;
            if let Some(i) = chips(app, ui, cwid, &bl, |i| i == bg, "mobile.comp.bg", u) {
                f.bg = i;
            }
            ui.add_space(24.0 * u);
        });
    });

    if create {
        let params = json!({
            "name": f.name.trim(),
            "width": f.width.trim().parse::<u32>().unwrap_or(1080),
            "height": f.height.trim().parse::<u32>().unwrap_or(1920),
            "frameRate": f.rate.trim().parse::<f64>().unwrap_or(30.0),
            "duration": f.secs.trim().parse::<f64>().unwrap_or(15.0),
            "background": BACKGROUNDS[f.bg].1,
        });
        let started = if project { app.session.execute("file.newProject", json!({})).map(|_| ()).map_err(|e| e.to_string()) } else { Ok(()) };
        match started.and_then(|_| app.session.execute("comp.new", params).map(|_| ()).map_err(|e| e.to_string())) {
            Ok(()) => result = Some(true),
            Err(e) => app.ui.status = format!("Cannot create the composition: {e}"),
        }
    }
    match result {
        Some(_) => {
            ctx.data_mut(|d| d.remove::<Form>(fid));
        }
        None => {
            ctx.data_mut(|d| d.insert_temp(fid, f));
        }
    }
    result
}

// ---------------------------------------------------------------------------------------------
// Leaving the editor with unsaved changes
// ---------------------------------------------------------------------------------------------

/// The "Save changes?" prompt. `Some(true)`: go Home (saved or discarded); `Some(false)`: stay.
pub fn confirm_leave(app: &mut EffectcraftApp, ctx: &egui::Context, full: Rect, u: f32) -> Option<bool> {
    let t = app.tokens;
    let mut out: Option<bool> = None;
    let dismissed = kit::scrim(ctx, "m-leave-scrim", full, 160);
    if dismissed {
        return Some(false);
    }
    let w = (full.width() - 40.0 * u).min(360.0 * u);
    let card = Rect::from_center_size(full.center(), vec2(w, 232.0 * u));
    let mut save = false;
    kit::overlay(ctx, "m-leave", card, |ui| {
        ui.painter().rect_filled(card, 20.0 * u, t.panel_bg);
        ui.painter().rect_stroke(card, 20.0 * u, Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
        ui.painter().text(pos2(card.center().x, card.min.y + 34.0 * u), Align2::CENTER_CENTER, "Save changes?", Tokens::semibold(18.0), t.text);
        ui.painter().text(pos2(card.center().x, card.min.y + 64.0 * u), Align2::CENTER_CENTER, "Your project has unsaved changes.", Tokens::ui(13.0), t.text_dim);
        let bw = w - 40.0 * u;
        let x = card.min.x + 20.0 * u;
        if kit::button(app, ui, Rect::from_min_size(pos2(x, card.min.y + 88.0 * u), vec2(bw, 44.0 * u)), "Save", Btn::Primary, true, "mobile.leave.save", u) {
            save = true;
        }
        let half = (bw - 10.0 * u) / 2.0;
        if kit::button(app, ui, Rect::from_min_size(pos2(x, card.min.y + 140.0 * u), vec2(half, 44.0 * u)), "Don't save", Btn::Danger, true, "mobile.leave.discard", u) {
            out = Some(true);
        }
        if kit::button(app, ui, Rect::from_min_size(pos2(x + half + 10.0 * u, card.min.y + 140.0 * u), vec2(half, 44.0 * u)), "Cancel", Btn::Secondary, true, "mobile.leave.cancel", u) {
            out = Some(false);
        }
    });
    if save {
        match save_project(app, false) {
            Ok(p) => {
                app.ui.status = format!("Saved to {p}");
                out = Some(true);
            }
            Err(e) => app.ui.status = format!("Not saved: {e}"),
        }
    }
    out
}
