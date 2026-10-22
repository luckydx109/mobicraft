# 04. Adaptive Preview & Effect Quality LOD

## 1. Preview Resolutions & Scaling Strategy

Motion graphics creators working on cinematic 9:16 vertical reels (1080x1920) need fluid 30–60 fps playback while scrubbing the timeline. Rendering 1080x1920 continuously during editing consumes immense battery and causes dropped frames on mobile GPUs.

The preview engine decouples **Viewport Resolution** from **Project Resolution**:

```
+-------------------------------------------------------------------------------+
| Preset Mode     | Resolution Scale | Vertical Reel Size (9:16) | Pixel Count  |
+-----------------+------------------+---------------------------+--------------+
| 360p (UltraFast)| 0.333x           | 360 x 640                 | 230,400 px   |
| 480p (Balanced) | 0.444x           | 480 x 854                 | 409,920 px   |
| 720p (Crisp)    | 0.667x           | 720 x 1280                | 921,600 px   |
| 1080p (Full)    | 1.000x           | 1080 x 1920               | 2,073,600 px |
+-------------------------------------------------------------------------------+
```

* **360p vs 1080p:** 360p requires **9x fewer pixel calculations** per frame than 1080p!
* On a 4 GB RAM budget phone (Helio G85), 360p allows 5–10 layer compositions with blend modes to play in real-time without dropping frames.
* The phone screen upsamples the preview texture via GPU bilinear hardware filtering; UI transform handles and vector outlines remain crisp at screen resolution.

---

## 2. Independence of Preview Quality vs Export Quality

In EffectCraft's architecture, preview and export use separate parameter pipelines:
1. **Interactive Viewer:** Governed by `RenderOpts` ([crates/render/src/lib.rs:387-412](file:///E:/effectcraft-mainformobile/crates/render/src/lib.rs#L387-L412)):
   * `scale`: Configurable by the user (360p, 480p, 720p).
   * `draft: true`: Disables multi-pass shadow calculations and clamps sample counts.
   * `motion_blur: false` (or draft 2-sample mode during playback).
2. **Final Render Queue:** Governed by `RenderSettings` ([crates/project/src/render_queue.rs:330-367](file:///E:/effectcraft-mainformobile/crates/project/src/render_queue.rs#L330-L367)):
   * `resolution`: Defaults to `1.0` (Full 1080p or 4K).
   * `quality`: `RenderQuality::Best`.
   * `motion_blur`: Full shutter angle sub-sampling (8–16 samples).

> **Result:** A user on a $150 smartphone can edit comfortably at 360p/480p with instant responsiveness, yet export a broadcast-quality 1080p MP4 reel.

---

## 3. Effect Level-of-Detail (LOD) Architecture

In [crates/effects/src/lib.rs:350-370](file:///E:/effectcraft-mainformobile/crates/effects/src/lib.rs#L350-L370), `EffectEnv` provides environmental parameters to effects.

### 3.1 Adding `EffectQuality` to `EffectEnv`
```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectQuality {
    Draft,      // Interactive scrubbing: lowest iterations, clamped limits
    Balanced,   // Moderate fidelity: 50% sample passes
    Full,       // Final export: highest precision passes
}
```

### 3.2 Effect Optimization Rules

#### A. Gaussian Blur & Directional Blur ([crates/effects/src/blur2.rs](file:///E:/effectcraft-mainformobile/crates/effects/src/blur2.rs))
* **Draft Mode:** Run 2-pass box blur approximation on a half-resolution working buffer.
* **Full Mode:** Full 5-pass high-precision recursive IIR filter.

#### B. Glow, Bloom & Light Rays ([crates/gpu/src/fx_stylize.rs](file:///E:/effectcraft-mainformobile/crates/gpu/src/fx_stylize.rs))
* **Draft Mode:** Single mip downscale (1/4 resolution) + additive threshold blend.
* **Full Mode:** 5-tier pyramid downscale with soft-knee threshold and bilateral blur.

#### C. GPU Particle Systems ([crates/gpu/src/particles.rs](file:///E:/effectcraft-mainformobile/crates/gpu/src/particles.rs))
* **Draft Mode:** Clamp active particle simulation budget to `max_particles = 250`.
* **Balanced Mode:** `max_particles = 750`.
* **Full Mode:** `max_particles = 3,000`.

---

## 4. Interactive Motion Blur Throttling

Motion blur calculates sub-frame positions by re-evaluating layer matrices multiple times per frame.
* **16-sample motion blur = 16x rendering cost per frame!**
* **Mobile Strategy:**
  1. While the user is dragging a layer on the canvas or scrubbing the timeline: `samples = 0` (Motion blur disabled in real-time interaction).
  2. During timeline playback: `samples = 2` (Fast linear motion smear).
  3. During final export: `samples = comp.shutter_samples` (Full cinematic 16-sample motion blur).
