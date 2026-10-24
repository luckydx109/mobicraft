//! The phone taps with a finger, not a mouse: egui-winit sends Touch events, then a pointer press
//! and release, and `PointerGone` as soon as the finger lifts (there is no hover on a touch
//! screen). These tests use that event sequence, so they catch buttons that only work for a mouse.
use effectcraft_engine::Session;
use effectcraft_engine::project::ItemId;
use effectcraft_ui_egui::{EffectcraftApp, mobile};
use egui::{Event, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

const W: f32 = 400.0;
const H: f32 = 860.0;

pub fn session() -> Session {
    let mut s = Session::default();
    let main = s.execute("comp.new", json!({"name": "Main Comp", "width": 1080, "height": 1920, "duration": 8})).unwrap()["comp"].as_u64().unwrap();
    s.state.open_comps = vec![ItemId(main)];
    s.state.active_comp = Some(ItemId(main));
    let _ = s.execute("layer.newSolid", json!({"name": "Background", "color": [0.10, 0.12, 0.30]}));
    let _ = s.execute("layer.newText", json!({"text": "EffectCraft"}));
    let _ = s.execute("layer.newNull", json!({}));
    s
}

fn phone(s: Session, size: egui::Vec2) -> Harness<'static, EffectcraftApp> {
    mobile::set_forced(true);
    let mut h = Harness::builder().with_size(size).build_eframe(move |_| {
        let mut app = EffectcraftApp::new(s);
        app.ui.start_screen = false;
        app
    });
    h.run_steps(10);
    h
}

fn id_rect(h: &Harness<'_, EffectcraftApp>, id: &str) -> Option<egui::Rect> {
    h.state().auto.previous.iter().chain(h.state().auto.elements.iter()).find(|e| e.id == id).map(|e| egui::Rect::from_min_size(pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3])))
}

fn touch_tap(h: &mut Harness<'_, EffectcraftApp>, p: Pos2) {
    let (dev, id) = (egui::TouchDeviceId(0), egui::TouchId(0));
    h.event(Event::Touch { device_id: dev, id, phase: egui::TouchPhase::Start, pos: p, force: None });
    h.event(Event::PointerMoved(p));
    h.event(Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.step();
    h.step();
    h.event(Event::Touch { device_id: dev, id, phase: egui::TouchPhase::End, pos: p, force: None });
    h.event(Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.event(Event::PointerGone);
    h.step();
    h.run_steps(8);
}

fn tap_id(h: &mut Harness<'_, EffectcraftApp>, id: &str) {
    let r = id_rect(h, id).unwrap_or_else(|| panic!("{id} is not on screen"));
    touch_tap(h, r.center());
}

fn touch_drag(h: &mut Harness<'_, EffectcraftApp>, from: Pos2, to: Pos2) {
    let (dev, id) = (egui::TouchDeviceId(0), egui::TouchId(0));
    h.event(Event::Touch { device_id: dev, id, phase: egui::TouchPhase::Start, pos: from, force: None });
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.step();
    for i in 1..=8 {
        let p = from + (to - from) * (i as f32 / 8.0);
        h.event(Event::Touch { device_id: dev, id, phase: egui::TouchPhase::Move, pos: p, force: None });
        h.event(Event::PointerMoved(p));
        h.step();
    }
    h.event(Event::Touch { device_id: dev, id, phase: egui::TouchPhase::End, pos: to, force: None });
    h.event(Event::PointerButton { pos: to, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.event(Event::PointerGone);
    h.step();
    h.run_steps(6);
}

fn comps(h: &Harness<'_, EffectcraftApp>) -> usize {
    h.state().session.project.comps().count()
}

#[test]
fn plus_next_to_the_comp_tabs_creates_a_composition() {
    let mut h = phone(session(), egui::vec2(W, H));
    let before = comps(&h);
    tap_id(&mut h, "mobile.nav.new");
    assert!(id_rect(&h, "mobile.comp.ok").is_some(), "the + opens the New Composition page");
    tap_id(&mut h, "mobile.comp.ok");
    assert_eq!(comps(&h), before + 1, "OK creates the composition");
    assert!(id_rect(&h, "mobile.comp.ok").is_none(), "and the page closes");
}

#[test]
fn camera_view_menu_opens_with_a_3d_layer() {
    let mut s = session();
    let comp = s.state.active_comp.unwrap().0;
    let _ = s.execute("layer.newCamera", json!({"comp": comp, "name": "Camera 1", "type": "twoNode"}));
    let layer = s.state.selected_layers.first().map(|l| l.0);
    let first = s.project.comp(effectcraft_engine::project::ItemId(comp)).map(|c| c.layers[0].id.0).or(layer).unwrap();
    let _ = s.execute("layer.setSwitch", json!({"comp": comp, "layers": [first], "switch": "threeD", "value": true}));
    let mut h = phone(s, egui::vec2(W, H));
    // The viewer's strip scrolls: slide it to the camera-view button.
    let y = id_rect(&h, "mobile.view.mag").unwrap().center().y;
    for _ in 0..8 {
        if id_rect(&h, "mobile.view.3d").is_some_and(|r| r.center().x > 0.0 && r.center().x < W) {
            break;
        }
        touch_drag(&mut h, pos2(360.0, y), pos2(40.0, y));
    }
    assert!(h.state().session.active_comp().unwrap().has_3d(), "the comp has a 3D layer, so the 3D views are enabled");
    tap_id(&mut h, "mobile.view.3d");
    let rows: Vec<String> = h.state().auto.previous.iter().filter(|e| e.id.starts_with("mobile.view.3d.")).map(|e| e.id.clone()).collect();
    assert!(rows.len() >= 6, "the camera-view menu lists the views (Active Camera, Front, Left, Top ...): {rows:?}");
    // Picking a view changes the comp's 3D view.
    let cid = h.state().session.active_comp_id().unwrap();
    let cur = |h: &Harness<'_, EffectcraftApp>| h.state().session.state.views3d.get(&cid).map(|v| v.current.id().to_string()).unwrap_or_default();
    let before = cur(&h);
    tap_id(&mut h, "mobile.view.3d.1");
    assert_ne!(cur(&h), before, "the picked view is applied");
}

/// Open a section of the + panel by its tab (sliding the strip as needed).
fn open_section(h: &mut Harness<'_, EffectcraftApp>, label: &str) {
    tap_id(h, "mobile.plus");
    let id = format!("mobile.plus.{label}");
    let y = id_rect(h, "mobile.plus.Info").or_else(|| id_rect(h, "mobile.plus.Assets")).map(|r| r.center().y).expect("the + panel's tab strip");
    for _ in 0..14 {
        let Some(r) = id_rect(h, &id) else { break };
        if r.min.x >= 0.0 && r.max.x <= W + 1.0 {
            break;
        }
        if r.min.x < 0.0 {
            touch_drag(h, pos2(60.0, y), pos2(360.0, y));
        } else {
            touch_drag(h, pos2(360.0, y), pos2(60.0, y));
        }
    }
    tap_id(h, &id);
}

#[test]
fn fill_and_stroke_section_edits_the_tool_paint() {
    use effectcraft_engine::commands::shape_tool::PaintKind;
    let mut h = phone(session(), egui::vec2(W, H));
    open_section(&mut h, "Fill & Stroke");
    for id in ["mobile.style.fill.kind.none", "mobile.style.fill.swatch", "mobile.style.stroke.kind.solid", "mobile.style.fill.opacity.inc", "mobile.style.fill.blend"] {
        let r = id_rect(&h, id).unwrap_or_else(|| panic!("{id} is on the panel"));
        assert!(r.min.x >= 0.0 && r.max.x <= W + 1.0, "{id} must not be cut off at the sides: {r:?}");
    }
    // Fill: None switches it off, Solid brings it back.
    assert_eq!(h.state().session.state.shape_tool.fill.kind, PaintKind::Solid);
    tap_id(&mut h, "mobile.style.fill.kind.none");
    assert_eq!(h.state().session.state.shape_tool.fill.kind, PaintKind::None, "tapping None switches the fill off");
    tap_id(&mut h, "mobile.style.fill.kind.solid");
    assert_eq!(h.state().session.state.shape_tool.fill.kind, PaintKind::Solid);
    // Stroke: kind and width. The panel scrolls: slide it up to reach the lower cards.
    let slide_to = |h: &mut Harness<'_, EffectcraftApp>, id: &str| {
        for _ in 0..8 {
            let r = id_rect(h, id).unwrap_or_else(|| panic!("{id} is on the panel"));
            if r.center().y < H - 60.0 && r.center().y > 300.0 {
                return;
            }
            touch_drag(h, pos2(200.0, 720.0), pos2(200.0, 380.0));
        }
        panic!("{id} never scrolled into view");
    };
    slide_to(&mut h, "mobile.style.stroke.kind.solid");
    tap_id(&mut h, "mobile.style.stroke.kind.solid");
    assert_eq!(h.state().session.state.shape_tool.stroke.kind, PaintKind::Solid);
    let w0 = h.state().session.state.shape_tool.stroke_width;
    slide_to(&mut h, "mobile.style.strokeWidth.inc");
    tap_id(&mut h, "mobile.style.strokeWidth.inc");
    assert!(h.state().session.state.shape_tool.stroke_width > w0, "the + steps the stroke width up");
    slide_to(&mut h, "mobile.style.strokeWidth.8");
    tap_id(&mut h, "mobile.style.strokeWidth.8");
    assert_eq!(h.state().session.state.shape_tool.stroke_width, 8.0);
    // Opacity of the fill.
    let o0 = h.state().session.state.shape_tool.fill.opacity;
    for _ in 0..8 {
        if id_rect(&h, "mobile.style.fill.opacity.dec").is_some_and(|r| r.center().y > 540.0 && r.center().y < H - 60.0) {
            break;
        }
        touch_drag(&mut h, pos2(200.0, 560.0), pos2(200.0, 850.0));
    }
    let dec = id_rect(&h, "mobile.style.fill.opacity.dec").unwrap();
    tap_id(&mut h, "mobile.style.fill.opacity.dec");
    assert!(h.state().session.state.shape_tool.fill.opacity < o0, "the - lowers the fill opacity (button at {dec:?}, opacity {o0} -> {})", h.state().session.state.shape_tool.fill.opacity);
}
