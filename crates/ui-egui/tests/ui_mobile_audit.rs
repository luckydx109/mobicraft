//! Dead-button detector for the phone UI. Run by hand (it renders every state with a software GPU):
//!   VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p effectcraft-ui-egui \
//!       --test ui_mobile_audit -- --ignored --nocapture
//! For every button on screen in several states it taps with a finger (Touch + press + release +
//! PointerGone, as a phone sends them) and compares a screenshot with an untouched control run.
//! Same picture = the button did nothing visible.
use effectcraft_engine::Session;
use effectcraft_engine::project::ItemId;
use effectcraft_ui_egui::{EffectcraftApp, mobile};
use egui::{Event, Pos2, pos2};
use egui_kittest::Harness;
use serde_json::json;

const W: f32 = 400.0;
const H: f32 = 860.0;

fn session() -> Session {
    let mut s = Session::default();
    let main = s.execute("comp.new", json!({"name": "Main Comp", "width": 1080, "height": 1920, "duration": 8})).unwrap()["comp"].as_u64().unwrap();
    s.state.open_comps = vec![ItemId(main)];
    s.state.active_comp = Some(ItemId(main));
    let _ = s.execute("layer.newSolid", json!({"name": "Background", "color": "#204080"}));
    let _ = s.execute("layer.newShape", json!({"name": "Shape"}));
    let _ = s.execute("layer.newText", json!({"text": "EffectCraft"}));
    let id = s.active_comp().unwrap().layers[0].id;
    let _ = s.execute("prop.addKey", json!({"layer": id.0, "path": "transform/opacity", "time": 0.0, "value": 0.0}));
    let _ = s.execute("prop.addKey", json!({"layer": id.0, "path": "transform/opacity", "time": 1.0, "value": 100.0}));
    let _ = s.execute("layer.select", json!({"layers": [id.0]}));
    s
}

fn phone(size: egui::Vec2) -> Harness<'static, EffectcraftApp> {
    mobile::set_forced(true);
    let s = session();
    let mut h = Harness::builder().with_size(size).build_eframe(move |_| {
        let mut app = EffectcraftApp::new(s);
        app.ui.start_screen = false;
        app
    });
    h.run_steps(10);
    for phase in [egui::TouchPhase::Start, egui::TouchPhase::End] {
        h.event(Event::Touch { device_id: egui::TouchDeviceId(0), id: egui::TouchId(0), phase, pos: pos2(1.0, 1.0), force: None });
        h.step();
    }
    h
}

fn rect_of(h: &Harness<'_, EffectcraftApp>, id: &str) -> Option<egui::Rect> {
    h.state().auto.previous.iter().find(|e| e.id == id).map(|e| egui::Rect::from_min_size(pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3])))
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

fn tap_id(h: &mut Harness<'_, EffectcraftApp>, id: &str) {
    let r = rect_of(h, id).unwrap_or_else(|| panic!("{id} not on screen"));
    touch_tap(h, r.center());
}

fn shot(h: &mut Harness<'_, EffectcraftApp>) -> Vec<u8> {
    h.render().unwrap().into_raw()
}

/// The + panel on `label`.
fn plus_section(h: &mut Harness<'_, EffectcraftApp>, label: &str) {
    tap_id(h, "mobile.plus");
    let id = format!("mobile.plus.{label}");
    let y = rect_of(h, "mobile.plus.Assets").or_else(|| rect_of(h, "mobile.plus.Info")).map(|r| r.center().y).unwrap_or(500.0);
    for _ in 0..16 {
        let Some(r) = rect_of(h, &id) else { return };
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

type Setup = Box<dyn Fn(&mut Harness<'static, EffectcraftApp>)>;

fn candidates(h: &Harness<'_, EffectcraftApp>) -> Vec<(String, egui::Rect)> {
    let mut v: Vec<(String, egui::Rect)> = h.state().auto.previous.iter().map(|e| (e.id.clone(), egui::Rect::from_min_size(pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3])))).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.dedup_by(|a, b| a.0 == b.0);
    v.retain(|(id, r)| {
        let c = r.center();
        !(id.starts_with("panel.") || id.contains(".row") || id == "timeline.header" || id.contains("scroll") || r.width() > 330.0 || r.height() > 160.0 || r.width() < 6.0 || r.height() < 6.0 || c.x <= 1.0 || c.x >= W - 1.0 || c.y <= 1.0 || c.y >= H - 1.0)
    });
    v
}

#[test]
#[ignore]
fn dead_buttons() {
    let mut states: Vec<(String, Setup)> = vec![("editor".into(), Box::new(|_| {}))];
    for label in ["Assets", "Effects", "Effect Settings", "Preview", "Properties", "Fill & Stroke", "Align", "Audio", "Character", "Paragraph", "History", "Markers", "Info", "Layer", "Brushes", "Wiggler", "Smoother", "Motion Sketch", "Mask Interpolation", "Footage", "Create Nulls", "Flowchart", "Render Queue", "Essential Graphics", "Paint & Brushes"] {
        let l = label.to_string();
        states.push((format!("+ {label}"), Box::new(move |h| plus_section(h, &l))));
    }
    states.push(("layer sheet".into(), Box::new(|h| {
        let id = h.state().session.active_comp().unwrap().layers[0].id.0;
        mobile::open_layer_menu(&h.ctx, id);
        h.run_steps(8);
    })));
    states.push(("import page".into(), Box::new(|h| tap_id(h, "mobile.import"))));

    let mut report = String::new();
    for (name, setup) in &states {
        let mut base = phone(egui::vec2(W, H));
        setup(&mut base);
        base.run_steps(20);
        let list = candidates(&base);
        let a = shot(&mut base);
        let b = shot(&mut base);
        let steady = a == b;
        let mut dead = vec![];
        for (id, r) in &list {
            let mut h = phone(egui::vec2(W, H));
            setup(&mut h);
            h.run_steps(20);
            let before = shot(&mut h);
            touch_tap(&mut h, r.center());
            let after = shot(&mut h);
            if before == after {
                dead.push(id.clone());
            }
        }
        println!("[{name}] {} elements, steady={steady}, dead: {dead:?}", list.len());
        report.push_str(&format!("[{name}] {dead:?}\n"));
    }
    println!("\n==== SUMMARY ====\n{report}");
}
