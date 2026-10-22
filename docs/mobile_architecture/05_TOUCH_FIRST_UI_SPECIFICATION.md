# 05. Touch-First Mobile UI Specification

## 1. Ergonomic Mobile Layout

Mobile video creators work single-handed or with thumbs in portrait mode. The desktop multi-dock workspace ([dock.rs](file:///E:/effectcraft-mainformobile/crates/ui-egui/src/dock.rs)) is replaced with a streamlined **4-Zone Mobile Editor**:

```
+-------------------------------------------------------------+
| Zone 1: Top Navigation Bar                                  |
| [Back]   [Undo] [Redo]   [360p/720p]   [Snap]     [Export]  |
+-------------------------------------------------------------+
| Zone 2: Viewport (Gesture Canvas)                           |
|                                                             |
|                   [9:16 Cinematic Canvas]                   |
|                   - Pinch-to-zoom / Pan                     |
|                   - Transform Gizmo Handles                 |
|                   - Social Media Safe Zone Guides           |
|                                                             |
+-------------------------------------------------------------+
| Zone 3: Quick Toolbelt                                      |
|  [+ Layer]  [✂ Split]  [📋 Duplicate]  [◇ Keyframe]  [🗑 Del] |
+-------------------------------------------------------------+
| Zone 4: Touch Timeline & Bottom Sheet Inspector             |
|                                                             |
|  [00:04.12] ◀ [Play/Pause] ▶              [Zoom - | +]     |
|  === Layer 3 (Text: "TITLE") ====== [◇] [◇] =============== |
|  === Layer 2 (Shape: Glow Burst) == [◇] ======= [◇] ======= |
|  === Layer 1 (Video: clip.mp4) ============================ |
|                                                             |
|  [Bottom Sheet Drawer: Transform | Effects | Blending | Curve]
+-------------------------------------------------------------+
```

---

## 2. Touch Gestures & Viewport Interactions

### 2.1 Canvas Gestures
* **Two-Finger Pinch:** Zooms the composition canvas into/out of view.
* **Two-Finger Drag:** Pans the canvas around the screen.
* **Single-Finger Tap:** Selects the topmost visible layer under the touch point.
* **Double Tap:** Resets canvas zoom to "Fit Screen" (100% view).

### 2.2 Transform Gizmo
When a layer is selected, a bounding box appears with touch-friendly handles (minimum touch target: `44x44 dp`):
* **Corner Circles:** Proportional scale.
* **Edge Squares:** Non-uniform stretch (width or height).
* **Top Arc Handle:** Rotation around the anchor point.
* **Center Crosshair:** Anchor point repositioning.

---

## 3. Touch Timeline & Keyframe Scrubbing

### 3.1 Timeline Gestures
* **Horizontal Drag:** Frame-accurate playhead scrubbing with haptic feedback ticks on whole seconds.
* **Pinch on Timeline:** Zooms time scale (from frame-level precision to multi-minute overview).
* **Layer Edge Drag:** Adjusts `in_point` and `out_point` with magnetic snapping to other layer cuts.

### 3.2 Keyframe System
* **Add/Remove Keyframe Button (Diamond `◇`):** Located next to the playhead.
  * Tapping adds a keyframe at the current `Tick` on the active property.
  * Solid diamond `◆` indicates a keyframe exists at the exact current frame; tapping toggles or edits it.
* **Keyframe Touch Graph Editor:**
  * Swiping up the "Curve" tab reveals a simplified Bezier graph.
  * Presets: *Linear*, *Easy Ease (33%)*, *Fast Out (Slow In)*, *Overshoot/Bounce*.
  * Two draggable tangent control circles adjust easing curves visually.

---

## 4. Bottom Sheet Inspector (Drawers)

Swiping upward opens the property inspector for the selected layer:

### Tab 1: Transform
* Sliders and step controls for:
  * **Position:** X, Y coordinates.
  * **Scale:** Uniform link toggle, percentage scale.
  * **Rotation:** Degrees dial.
  * **Opacity:** 0% to 100%.

### Tab 2: Effects & Styles
* Add Effect button (`+ FX`): Displays searchable categories (Blur, Glow, Color, Distort).
* Reorderable effect stack.
* Live parameter sliders (e.g. *Glow Radius*, *Glow Intensity*, *Vignette Softness*).

### Tab 3: Blending & Mattes
* Dropdown for all 38 blend modes (*Normal, Screen, Multiply, Overlay, Add, Soft Light, etc.*).
* Track Matte selector (*Alpha Matte, Inverted Alpha, Luma Matte*).

---

## 5. Built-In Cinematic Reel Presets

To empower quick creation of professional social media content, the editor integrates pre-built parameter templates:
1. **Dynamic Text Reveals:** Typewriter, Kinetic Bounce, Glitch Text, Blur Drop.
2. **Smooth Speed Ramping:** Fast-motion to Slow-mo Bezier curve presets with audio pitch preservation.
3. **Cinematic Transitions:** Whip Pan, Zoom Blur, RGB Split Glitch, Light Leak Flash.
4. **Camera Shake & Wiggle:** Procedural noise expressions (`wiggle(freq, amp)`) simulating handheld camera movement.
