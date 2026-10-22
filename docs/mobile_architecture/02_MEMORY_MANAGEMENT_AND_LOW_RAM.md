# 02. Memory Management & Low-RAM Adaptation

## 1. Physical RAM Tiers & App Memory Budgets

Android uses unified memory (RAM is shared between CPU and GPU). If an app exceeds its allocated memory envelope, the Android Low Memory Killer (LMK) sends `SIGKILL` without catchable panic unwinding.

The memory management system divides Android devices into 4 hardware tiers:

```
+-----------------------------------------------------------------------------------------+
| Tier          | Physical RAM | App RAM Ceiling | RAM Preview Cache | Texture Working Pool | Max CPU Threads |
+---------------+--------------+-----------------+-------------------+----------------------+-----------------+
| Tier 1 (Low)  | 4 GB         | 350 MB          | 48 MB             | 64 MB (Rgba8Unorm)   | 2 threads       |
| Tier 2 (Mid)  | 6 GB         | 650 MB          | 96 MB             | 128 MB (Rgba8Unorm)  | 3 threads       |
| Tier 3 (High) | 8 GB         | 1.2 GB          | 256 MB            | 256 MB (Rgba16Float) | 4 threads       |
| Tier 4 (Ultra)| 12 GB+       | 2.5 GB          | 512 MB            | 512 MB (Rgba32Float) | 6–8 threads     |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Dynamic Cache Budgeting

### 2.1 Refactoring `Prefs::cache_budgets()`
In [crates/engine/src/prefs.rs:589-609](file:///E:/effectcraft-mainformobile/crates/engine/src/prefs.rs#L589-L609), the desktop logic has a hardcoded floor of 64 MB per category (`3 * 64 MB = 192 MB` minimum). On a 4 GB Android device, this hard floor alone consumes more than half of the safe memory ceiling.

**Required Mobile Modification:**
```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceTier {
    Tier4Gb,
    Tier6Gb,
    Tier8Gb,
    Tier12GbPlus,
}

impl DeviceTier {
    pub fn from_total_ram(bytes: u64) -> Self {
        match bytes {
            b if b <= 4_500_000_000 => DeviceTier::Tier4Gb,
            b if b <= 6_500_000_000 => DeviceTier::Tier6Gb,
            b if b <= 9_000_000_000 => DeviceTier::Tier8Gb,
            _ => DeviceTier::Tier12GbPlus,
        }
    }

    pub fn floor_bytes(self) -> usize {
        match self {
            DeviceTier::Tier4Gb => 16 << 20,       // 16 MB floor
            DeviceTier::Tier6Gb => 32 << 20,       // 32 MB floor
            DeviceTier::Tier8Gb => 48 << 20,       // 48 MB floor
            DeviceTier::Tier12GbPlus => 64 << 20,  // 64 MB floor
        }
    }
}
```

### 2.2 LRU Frame Eviction & Buffer Recycling
In [crates/media/src/pool.rs:83-134](file:///E:/effectcraft-mainformobile/crates/media/src/pool.rs#L83-L134):
* `spare: Vec<Vec<Px>>`: Recycles evicted pixel buffers up to `MAX_SPARE`.
* Eviction runs on byte thresholds rather than frame counts, ensuring odd aspect ratios or vertical reel dimensions (1080x1920) do not overshoot allocations.

---

## 3. Working Texture Memory Optimization

### 3.1 Problem: Hardcoded `Rgba32Float`
In [crates/gpu/src/context.rs:42](file:///E:/effectcraft-mainformobile/crates/gpu/src/context.rs#L42):
```rust
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
```
* Single 1080x1920 texture at FP32 = `1920 * 1080 * 16 bytes = ~33.1 MB`.
* A 5-layer composition with blending creates multiple intermediate textures, reaching **150+ MB of VRAM for a single preview frame**.
* Mobile GPUs (Mali-G52, Adreno 610) throttle severely due to memory bus saturation on 32-bit floating-point textures.

### 3.2 Solution: Dynamic Texture Format by Tier
```rust
pub fn select_texture_format(tier: DeviceTier) -> wgpu::TextureFormat {
    match tier {
        DeviceTier::Tier4Gb | DeviceTier::Tier6Gb => wgpu::TextureFormat::Rgba8Unorm, // 4 bytes/px (75% savings)
        DeviceTier::Tier8Gb => wgpu::TextureFormat::Rgba16Float,                      // 8 bytes/px (50% savings, HDR headroom)
        DeviceTier::Tier12GbPlus => wgpu::TextureFormat::Rgba32Float,                 // 16 bytes/px (Studio precision)
    }
}
```

---

## 4. Handling Android Memory Pressure (`onTrimMemory`)

Android signals app memory pressure via `ComponentCallbacks2.onTrimMemory(level)`. The engine must listen and respond immediately.

### 4.1 Memory Pressure Event Mapping
```rust
#[repr(i32)]
pub enum AndroidTrimLevel {
    RunningModerate = 5,
    RunningLow = 10,
    RunningCritical = 15,
    UiHidden = 20,
    Background = 40,
    Moderate = 60,
    Complete = 80,
}

impl Session {
    pub fn on_trim_memory(&mut self, level: i32) {
        match level {
            // Background / Complete: App is hidden, release all display textures
            20..=80 => {
                log::info!("Android trim memory: UI hidden/background -> full purge");
                self.purge_all_caches();
            }
            // Running Critical / Running Low: Foreground is in danger of being killed
            10..=15 => {
                log::warn!("Android trim memory: critical -> halving cache budgets");
                self.shrink_cache_budgets(0.5);
                self.purge_spare_buffers();
            }
            // Moderate: Gentle cleanup
            5 => {
                self.evict_stale_frames();
            }
            _ => {}
        }
    }
}
```

---

## 5. CPU Threadpool Bounding (Rayon)

### 5.1 Preventing Thermal Over-Subscription
Android SoCs feature asymmetric CPU clusters (e.g. 2 Cortex-A76 performance cores + 6 Cortex-A55 efficiency cores). Spawning 8 threads causes the efficiency cores to drag down synchronization barriers, causing stutter and thermal throttling.

**Configuration Rule:**
```rust
pub fn configure_mobile_rayon(tier: DeviceTier) {
    let threads = match tier {
        DeviceTier::Tier4Gb => 2,
        DeviceTier::Tier6Gb => 3,
        DeviceTier::Tier8Gb => 4,
        DeviceTier::Tier12GbPlus => 6,
    };

    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("ec-worker-{i}"))
        .build_global();
}
```
This preserves 2+ cores exclusively for the Android OS UI thread and audio playback thread, eliminating dropped audio and UI freezing during timeline scrubbing.
