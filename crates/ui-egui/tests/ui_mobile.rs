//! The phone UI (egui_kittest): the editor opens with everything closed, lays out inside the
//! screen with finger-sized controls in portrait and landscape, and the main flows work: Home to
//! New Project to the editor, menus (without the removed tracking features), sub-tools, the +
//! panel, the layer sheet, playback, saving and the export page.

use effectcraft_engine::Session;
use effectcraft_engine::project::ItemId;
use effectcraft_ui_egui::EffectcraftApp;
use effectcraft_ui_egui::mobile;
use egui::{Event, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

fn session() -> Session {
    let mut s = Session::default();
    let main = s.execute("comp.new", json!({"name": "Main", "width": 320, "height": 180, "duration": 4})).unwrap()["comp"].as_u64().unwrap();
    s.state.open_comps = vec![ItemId(main)];
    s.state.active_comp = Some(ItemId(main));
    s.execute("layer.newSolid", json!({"name": "Back", "color": [0.2, 0.3, 0.8]})).unwrap();
    s.execute("layer.newNull", json!({})).unwrap();
    s
}

fn harness_with(s: Session, size: egui::Vec2, home: bool) -> Harness<'static, EffectcraftApp> {
    mobile::set_forced(true);
    let mut h = Harness::builder().with_size(size).build_eframe(move |_| {
        let mut app = EffectcraftApp::new(s);
        app.ui.start_screen = home;
        app
    });
    h.run_steps(4);
    h
}

fn harness(size: egui::Vec2) -> Harness<'static, EffectcraftApp> {
    harness_with(session(), size, false)
}

fn find(h: &Harness<'_, EffectcraftApp>, id: &str) -> Option<egui::Rect> {
    h.state().auto.previous.iter().chain(h.state().auto.elements.iter()).find(|e| e.id == id).map(|e| egui::Rect::from_min_size(pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3])))
}

fn exists(h: &Harness<'_, EffectcraftApp>, prefix: &str) -> bool {
    h.state().auto.previous.iter().chain(h.state().auto.elements.iter()).any(|e| e.id.starts_with(prefix))
}

fn tap_at(h: &mut Harness<'_, EffectcraftApp>, p: Pos2) {
    h.event(Event::PointerMoved(p));
    h.step();
    h.event(Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.step();
    h.event(Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.step();
    h.run_steps(3);
}

fn tap(h: &mut Harness<'_, EffectcraftApp>, id: &str) {
    let p = find(h, id).unwrap_or_else(|| panic!("no element {id}")).center();
    tap_at(h, p);
}

fn check_inside(h: &Harness<'_, EffectcraftApp>, screen: egui::Rect, ids: &[&str]) {
    for id in ids {
        let r = find(h, id).unwrap_or_else(|| panic!("no element {id}"));
        assert!(screen.expand(0.5).contains_rect(r), "{id} {r:?} is outside the screen {screen:?}");
        assert!(r.width() >= 36.0 && r.height() >= 36.0, "{id} {r:?} is too small to touch");
    }
}

const TOOLBAR: [&str; 7] = ["mobile.toolbar", "mobile.home", "mobile.import", "mobile.options", "mobile.undo", "mobile.redo", "mobile.export"];
const TOOLS: [&str; 7] = ["mobile.tool.Selection", "mobile.tool.Hand", "mobile.tool.Rotation", "mobile.tool.Anchor", "mobile.tool.Shape", "mobile.tool.Pen", "mobile.tool.Text"];

#[test]
fn editor_opens_with_everything_closed_in_portrait() {
    let h = harness(egui::vec2(400.0, 860.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    check_inside(&h, screen, &TOOLBAR);
    check_inside(&h, screen, &TOOLS);
    check_inside(&h, screen, &["mobile.view.mag", "mobile.view.grid", "mobile.nav.new", "mobile.plus", "mobile.play"]);
    // Nothing is open: no menu strip, popup, sheet, + panel or page.
    for closed in ["mobile.menu.", "mobile.view.mag.0", "mobile.layer.", "mobile.plus.", "mobile.export.start", "mobile.import", "mobile.tool.Shape.", "mobile.comp."] {
        if closed == "mobile.import" {
            continue; // the toolbar's own Import button
        }
        assert!(!exists(&h, closed), "{closed} must be closed at start");
    }
    // Order: toolbar, preview, strip, navigation, timeline.
    let top = find(&h, "mobile.home").unwrap();
    let pv = find(&h, "panel.Composition").unwrap();
    let vb = find(&h, "mobile.view.mag").unwrap();
    let nav = find(&h, "mobile.nav.new").unwrap();
    let tl = find(&h, "panel.Timeline").unwrap();
    assert!(top.max.y <= pv.min.y + 50.0 && pv.max.y <= vb.min.y + 1.0 && vb.max.y <= nav.min.y + 1.0 && nav.max.y <= tl.min.y + 20.0, "{top:?} {pv:?} {vb:?} {nav:?} {tl:?}");
    assert!(tl.height() > 250.0 && pv.height() > 150.0, "pv {pv:?} tl {tl:?}");
}

#[test]
fn landscape_lays_out_inside_the_screen() {
    let h = harness(egui::vec2(860.0, 400.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(860.0, 400.0));
    check_inside(&h, screen, &TOOLBAR);
    check_inside(&h, screen, &["mobile.plus"]);
    let pv = find(&h, "panel.Composition").unwrap();
    let tl = find(&h, "panel.Timeline").unwrap();
    assert!(pv.max.x <= tl.min.x + 1.0, "preview left of timeline: {pv:?} {tl:?}");
}

#[test]
fn playback_tools_and_subtools() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    tap(&mut h, "mobile.play");
    assert!(h.state().playback.playing, "play starts playback");
    tap(&mut h, "mobile.play");
    assert!(!h.state().playback.playing, "and stops it");
    tap(&mut h, "mobile.tool.Hand");
    assert_eq!(h.state().ui.tool, effectcraft_ui_egui::state::Tool::Hand);
    // A slot with more tools: tap once to select, again for its sub-tools.
    tap(&mut h, "mobile.tool.Shape");
    assert_eq!(h.state().ui.tool, effectcraft_ui_egui::state::Tool::Rectangle);
    tap(&mut h, "mobile.tool.Shape");
    for star in ["Rectangle", "Rounded Rectangle", "Ellipse", "Polygon", "Star"] {
        assert!(find(&h, &format!("mobile.tool.Shape.{star}")).is_some(), "sub-tool {star}");
    }
    tap(&mut h, "mobile.tool.Shape.Star");
    assert_eq!(h.state().ui.tool, effectcraft_ui_egui::state::Tool::Star);
    assert!(find(&h, "mobile.tool.Shape.Star").is_none(), "the popup closes after a pick");
}

#[test]
fn options_menus_open_and_hide_removed_features() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    tap(&mut h, "mobile.options");
    for m in ["File", "Edit", "Composition", "Layer", "Effect", "Animation", "View", "Window", "Help"] {
        assert!(find(&h, &format!("mobile.menu.{m}")).is_some(), "menu {m}");
    }
    tap(&mut h, "mobile.menu.Animation");
    assert!(exists(&h, "mobile.menu.Animation."), "the Animation menu lists its entries");
    assert!(!exists(&h, "mobile.menu.Animation.track."), "tracking entries are not offered");
    // A tap outside closes the popup.
    tap_at(&mut h, pos2(200.0, 845.0));
    assert!(!exists(&h, "mobile.menu.Animation."), "a tap outside closes the menu");
}

#[test]
fn plus_panel_swaps_with_the_timeline_and_back() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    tap(&mut h, "mobile.plus");
    for s in ["Assets", "Effects & Presets", "Effect Settings", "Preview", "Properties", "Align", "Audio"] {
        assert!(find(&h, &format!("mobile.plus.{s}")).is_some(), "section {s}");
    }
    assert!(find(&h, "panel.Project").is_some(), "Assets shows the Project panel");
    tap(&mut h, "mobile.plus.Effects & Presets");
    assert!(find(&h, "panel.EffectsPresets").is_some());
    tap(&mut h, "mobile.plus.close");
    assert!(find(&h, "panel.Timeline").is_some() && !exists(&h, "mobile.plus."), "back to the timeline");
}

#[test]
fn layer_sheet_creates_layers_and_toggles_switches() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let id = h.state().session.active_comp().unwrap().layers[0].id.0;
    mobile::open_layer_menu(&h.ctx, id);
    h.run_steps(3);
    for need in ["mobile.layer.close", "mobile.layer.new.Text", "mobile.layer.new.Camera", "mobile.layer.op.edit.duplicate", "mobile.layer.group.Blending Mode", "mobile.layer.switch.motionBlur"] {
        assert!(find(&h, need).is_some(), "{need}");
    }
    let before = h.state().session.active_comp().unwrap().layers.len();
    // The sheet opens on the quick toggles, blending, matte, parent ...; Create is further down: slide
    // the sheet up (a finger drag) until it shows.
    enable_touch(&mut h);
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    let close = find(&h, "mobile.layer.close").unwrap();
    for _ in 0..8 {
        let text = find(&h, "mobile.layer.new.Text").unwrap();
        if text.min.y >= close.max.y && text.max.y <= screen.max.y - 40.0 {
            break;
        }
        drag(&mut h, pos2(200.0, 700.0), pos2(200.0, 330.0));
    }
    let text = find(&h, "mobile.layer.new.Text").unwrap();
    assert!(text.min.y >= close.max.y && text.max.y <= screen.max.y, "Create > Text slid into view: {text:?}");
    tap(&mut h, "mobile.layer.new.Text");
    assert_eq!(h.state().session.active_comp().unwrap().layers.len(), before + 1, "Create > Text adds a layer");
    assert!(!exists(&h, "mobile.layer."), "the sheet closes after an action");
}

#[test]
fn home_new_project_opens_the_editor_and_leaving_saves() {
    let dir = std::env::temp_dir().join(format!("ec-mobile-test-{}", std::process::id()));
    mobile::set_data_dir(&dir.to_string_lossy());
    let mut h = harness_with(Session::default(), egui::vec2(400.0, 860.0), true);
    for need in ["mobile.home.new", "mobile.home.open", "mobile.home.link.Discord"] {
        assert!(find(&h, need).is_some(), "{need}");
    }
    tap(&mut h, "mobile.home.new");
    assert!(find(&h, "mobile.comp.ok").is_some() && find(&h, "mobile.comp.preset.0").is_some(), "the composition settings page");
    tap(&mut h, "mobile.comp.ok");
    assert!(!h.state().ui.start_screen, "OK opens the editor");
    let c = h.state().session.active_comp().expect("a composition was created");
    assert_eq!((c.width, c.height), (1080, 1920), "the first preset is Shorts / Reels 9:16");
    // Unsaved work: Home asks, Save writes the project into the app folder and goes Home.
    h.state_mut().session.execute("layer.newNull", json!({})).unwrap();
    h.run_steps(2);
    tap(&mut h, "mobile.home");
    assert!(find(&h, "mobile.leave.save").is_some(), "unsaved changes are asked about");
    tap(&mut h, "mobile.leave.save");
    assert!(h.state().ui.start_screen, "back at Home");
    let saved: Vec<_> = std::fs::read_dir(dir.join("Projects")).map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect()).unwrap_or_default();
    assert!(saved.iter().any(|n| n.ends_with(".ecproj")), "the project file was written: {saved:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_page_shows_the_real_options() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    tap(&mut h, "mobile.export");
    assert!(find(&h, "mobile.export.start").is_some(), "the start button");
    assert!(find(&h, "mobile.export.back").is_some());
    tap(&mut h, "mobile.export.back");
    assert!(!exists(&h, "mobile.export.start"), "back closes the page");
}

// ---------------------------------------------------------------------------------------------
// System bars, the sliding timeline and the movable +
// ---------------------------------------------------------------------------------------------

/// A phone whose host reports `insets` (`[left, top, right, bottom]`, in pixels = points here).
fn harness_insets(size: egui::Vec2, insets: [f32; 4]) -> Harness<'static, EffectcraftApp> {
    mobile::set_forced(true);
    let s = session();
    let mut h = Harness::builder().with_size(size).build_eframe(move |_| {
        let mut app = EffectcraftApp::new(s);
        app.ui.start_screen = false;
        app.hooks.window_insets = Some(Box::new(move || Some(insets)));
        app
    });
    h.run_steps(6);
    h
}

/// A phone has a touch screen: egui scrolls lists by dragging their contents only once it has seen
/// a touch (`DragScroll::OnTouch`), which the tests' mouse events never tell it.
fn enable_touch(h: &mut Harness<'_, EffectcraftApp>) {
    for phase in [egui::TouchPhase::Start, egui::TouchPhase::End] {
        h.event(Event::Touch { device_id: egui::TouchDeviceId(0), id: egui::TouchId(0), phase, pos: pos2(2.0, 2.0), force: None });
        h.step();
    }
}

fn drag(h: &mut Harness<'_, EffectcraftApp>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.step();
    h.event(Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.step();
    for i in 1..=6 {
        let p = from + (to - from) * (i as f32 / 6.0);
        h.event(Event::PointerMoved(p));
        h.step();
    }
    h.event(Event::PointerButton { pos: to, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.step();
    h.run_steps(3);
}

#[test]
fn controls_stay_clear_of_the_system_bars_in_portrait_and_landscape() {
    // Portrait: a status bar on top, a 3-button navigation bar at the bottom.
    let (w, hgt) = (400.0, 860.0);
    let insets = [0.0, 28.0, 0.0, 96.0];
    let h = harness_insets(egui::vec2(w, hgt), insets);
    let safe = egui::Rect::from_min_max(pos2(0.0, insets[1]), pos2(w, hgt - insets[3]));
    check_inside(&h, safe, &TOOLBAR);
    check_inside(&h, safe, &TOOLS);
    check_inside(&h, safe, &["mobile.view.mag", "mobile.nav.new", "mobile.plus", "mobile.play", "mobile.timeline.pan"]);

    // Landscape: the navigation bar moves to the side and a notch takes the other.
    let (w, hgt) = (860.0, 400.0);
    let insets = [48.0, 0.0, 48.0, 24.0];
    let h = harness_insets(egui::vec2(w, hgt), insets);
    let safe = egui::Rect::from_min_max(pos2(insets[0], insets[1]), pos2(w - insets[2], hgt - insets[3]));
    check_inside(&h, safe, &TOOLBAR);
    check_inside(&h, safe, &["mobile.plus", "mobile.timeline.pan"]);
    let pv = find(&h, "panel.Composition").unwrap();
    let tl = find(&h, "panel.Timeline").unwrap();
    assert!(pv.max.x <= tl.min.x + 1.0 && safe.contains_rect(tl), "{pv:?} {tl:?} {safe:?}");
}

#[test]
fn the_whole_timeline_slides_sideways() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    let bar = find(&h, "mobile.timeline.pan").expect("a bar under the timeline");
    assert!(screen.contains_rect(bar) && bar.height() >= 36.0, "{bar:?}");
    let tl = find(&h, "panel.Timeline").unwrap();
    assert!(screen.contains_rect(tl) && tl.max.y <= bar.min.y + 1.0, "the timeline ends above the bar: {tl:?} {bar:?}");
    // It opens on the layer names next to the time graph; the other columns wait on the left.
    let name = find(&h, "timeline.header.name").expect("name column");
    assert!(screen.contains_rect(egui::Rect::from_min_size(name.min, egui::vec2(100.0, 20.0))), "names are on screen: {name:?}");
    let parent = find(&h, "timeline.header.parent").expect("Parent & Link column");
    assert!(parent.max.x <= name.min.x + 1.0 && parent.min.x < 0.0, "Parent & Link waits on the left: {parent:?} {name:?}");
    // Dragging the bar to its start slides the columns in: Switches and Parent & Link are on screen.
    drag(&mut h, pos2(bar.max.x - 40.0, bar.center().y), pos2(bar.min.x - 80.0, bar.center().y));
    for col in ["av", "label", "num", "switches", "parent"] {
        let r = find(&h, &format!("timeline.header.{col}")).unwrap_or_else(|| {
            let ids: Vec<String> = h.state().auto.previous.iter().chain(h.state().auto.elements.iter()).filter(|e| e.id.starts_with("timeline.header")).map(|e| format!("{} {:?}", e.id, e.rect)).collect();
            panic!("column {col}; registered: {ids:?}")
        });
        assert!(r.min.x >= -1.0 && r.max.x <= 400.0 + 1.0, "{col} must be fully on screen when slid in: {r:?}");
    }
    // Everything that stays put is still inside the screen: the bar and the +.
    check_inside(&h, screen, &["mobile.timeline.pan", "mobile.plus", "mobile.play"]);
    // And back to names + graph.
    drag(&mut h, pos2(bar.min.x + 40.0, bar.center().y), pos2(bar.max.x + 80.0, bar.center().y));
    let back = find(&h, "timeline.header.name").unwrap().min.x;
    assert!((back - name.min.x).abs() < 2.0, "{} -> {back}", name.min.x);
}

#[test]
fn the_columns_button_opens_a_sheet_with_every_column() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    check_inside(&h, screen, &["timeline.switch.columns", "timeline.switch.modes", "timeline.switch.search", "timeline.switch.graph"]);
    tap(&mut h, "timeline.switch.columns");
    h.run_steps(6);
    for col in ["av", "label", "num", "switches", "modes", "parent", "keys", "in", "out", "duration", "stretch", "comment"] {
        assert!(find(&h, &format!("mobile.col.toggle.{col}")).is_some(), "the column sheet must offer {col}");
    }
    assert!(h.state().ui.timeline.columns.contains("parent"), "Parent & Link is on by default");
    tap(&mut h, "mobile.col.toggle.parent");
    assert!(!h.state().ui.timeline.columns.contains("parent"), "the sheet switches the column off");
    tap(&mut h, "mobile.col.toggle.parent");
    assert!(h.state().ui.timeline.columns.contains("parent"), "and on again");
}

#[test]
fn the_timeline_strip_navigates_keyframes_and_toggles_modes() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    assert!(!h.state().ui.timeline.show_modes);
    tap(&mut h, "timeline.switch.modes");
    assert!(h.state().ui.timeline.show_modes, "the Modes chip shows the Modes columns");
    tap(&mut h, "timeline.switch.modes");
    // Twirl (past the right edge of a 400 pt phone: slide the strip) opens the selected layers'
    // properties, a second tap closes them.
    enable_touch(&mut h);
    let y = find(&h, "timeline.switch.columns").unwrap().center().y;
    drag(&mut h, pos2(360.0, y), pos2(60.0, y));
    let twirl = find(&h, "timeline.switch.twirl").unwrap();
    assert!(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0)).expand(1.0).contains_rect(twirl), "the strip slid to the Twirl chip: {twirl:?}");
    let id = h.state().session.active_comp().unwrap().layers[0].id.0;
    h.state_mut().session.state.selected_layers = vec![effectcraft_engine::project::LayerId(id)];
    h.run_steps(3);
    tap(&mut h, "timeline.switch.twirl");
    assert!(h.state().ui.timeline.open_layers.contains(&id), "twirl opens the selected layer");
    tap(&mut h, "timeline.switch.twirl");
    assert!(!h.state().ui.timeline.open_layers.contains(&id), "and closes it");
}

#[test]
fn layer_sheet_picks_a_parent() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let ids: Vec<u64> = h.state().session.active_comp().unwrap().layers.iter().map(|l| l.id.0).collect();
    let (child, parent) = (ids[0], ids[1]);
    mobile::open_layer_menu(&h.ctx, child);
    h.run_steps(8);
    tap(&mut h, "mobile.layer.parent.toggle");
    tap(&mut h, &format!("mobile.layer.parent.{parent}"));
    let got = h.state().session.active_comp().unwrap().layers.iter().find(|l| l.id.0 == child).and_then(|l| l.parent).map(|p| p.0);
    assert_eq!(got, Some(parent), "the sheet's parent picker sets the layer's parent");
}

#[test]
fn the_plus_can_be_moved_and_still_opens_the_panels() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    let start = find(&h, "mobile.plus").unwrap();
    drag(&mut h, start.center(), start.center() + egui::vec2(-150.0, -90.0));
    let moved = find(&h, "mobile.plus").unwrap();
    assert!(moved.center().x < start.center().x - 100.0 && moved.center().y < start.center().y - 60.0, "{start:?} -> {moved:?}");
    assert!(!exists(&h, "mobile.plus."), "dragging must not open the panel");
    check_inside(&h, screen, &["mobile.plus"]);
    // Dragged past the edge it stays on the screen.
    drag(&mut h, moved.center(), pos2(-80.0, -80.0));
    check_inside(&h, screen, &["mobile.plus"]);
    // A tap still opens the panel.
    tap(&mut h, "mobile.plus");
    assert!(find(&h, "mobile.plus.Assets").is_some(), "tap opens the + panel");
}

#[test]
fn the_plus_panel_offers_the_pc_panels_that_were_only_in_the_window_menu() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 860.0));
    enable_touch(&mut h);
    tap(&mut h, "mobile.plus");
    let y = find(&h, "mobile.plus.Info").expect("the strip").center().y;
    let before = find(&h, "mobile.plus.Info").unwrap();
    drag(&mut h, pos2(360.0, y), pos2(60.0, y));
    let after = find(&h, "mobile.plus.Info").unwrap();
    assert!(after.min.x < before.min.x - 100.0, "the strip must scroll by dragging: {before:?} -> {after:?}");
    for label in ["Paint & Brushes", "Brushes", "Layer", "Wiggler", "Smoother", "Motion Sketch", "Mask Interpolation", "Essential Graphics", "Footage", "Flowchart", "Create Nulls"] {
        let id = format!("mobile.plus.{label}");
        // Slide the strip until the tab shows (left or right, whichever way it lies).
        for _ in 0..14 {
            let Some(r) = find(&h, &id) else { break };
            if screen.expand(1.0).contains_rect(r) {
                break;
            }
            if r.min.x < 0.0 {
                drag(&mut h, pos2(60.0, y), pos2(360.0, y));
            } else {
                drag(&mut h, pos2(360.0, y), pos2(60.0, y));
            }
        }
        let tab = find(&h, &id).unwrap_or_else(|| panic!("a {label} tab"));
        assert!(screen.expand(1.0).contains_rect(tab), "{label} tab {tab:?}");
        tap_at(&mut h, tab.center());
        let body = h.state().auto.previous.iter().chain(h.state().auto.elements.iter()).filter(|e| e.id.starts_with("panel.")).map(|e| egui::Rect::from_min_size(pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3]))).max_by(|a, b| a.height().total_cmp(&b.height())).expect("a panel");
        assert!(screen.contains_rect(body) && body.height() > 150.0, "{label} panel {body:?}");
    }
}

#[test]
fn layer_sheet_quick_toggles_are_not_cut_off() {
    let mut h = harness(egui::vec2(400.0, 860.0));
    let id = h.state().session.active_comp().unwrap().layers[0].id.0;
    mobile::open_layer_menu(&h.ctx, id);
    h.run_steps(8);
    let close = find(&h, "mobile.layer.close").unwrap();
    // The sheet's visible width ends where its close button ends (same 12 pt margin on the left).
    for key in ["eye", "solo", "lock"] {
        let r = find(&h, &format!("mobile.layer.quick.{key}")).unwrap();
        assert!(r.min.x >= 11.0 && r.max.x <= close.max.x - 4.0, "{key} pill {r:?} must end inside the sheet (close button ends at {})", close.max.x);
    }
}
