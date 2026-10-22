# 03. Hardware-Accelerated Video Pipeline

## 1. Overview & Android Platform APIs

On mobile devices, pure-software video encoding on CPU is completely unviable for 1080p rendering. Android provides dedicated silicon blocks (DSP/hardware encoders) through the NDK:
* **Encoding:** `AMediaCodec` (`<media/NdkMediaCodec.h>`)
* **Decoding / Demuxing:** `AMediaExtractor` (`<media/NdkMediaExtractor.h>`)
* **Format Negotiation:** `AMediaFormat` (`<media/NdkMediaFormat.h>`)

EffectCraft integrates these NDK APIs directly into its Rust export and media import pipeline.

---

## 2. Encoder Integration (`VideoEncoder` Trait)

In [crates/export/src/encode.rs:29-37](file:///E:/effectcraft-mainformobile/crates/export/src/encode.rs#L29-L37), EffectCraft already defines an extensible trait:
```rust
pub(crate) trait VideoEncoder {
    fn sample_entry(&self) -> SampleEntry;
    fn encode(&mut self, rgba: &[u8], index: u64) -> Result<Vec<Packet>>;
    fn flush(&mut self) -> Result<Vec<Packet>>;
    fn media_start(&self) -> Option<i64> { None }
}
```

### 2.1 The `NdkMediaCodecEncoder` Implementation

```rust
use ndk_sys::{
    AMediaCodec, AMediaCodecBufferInfo, AMediaCodec_createEncoderByType,
    AMediaCodec_configure, AMediaCodec_start, AMediaCodec_dequeueInputBuffer,
    AMediaCodec_getInputBuffer, AMediaCodec_queueInputBuffer,
    AMediaCodec_dequeueOutputBuffer, AMediaCodec_getOutputBuffer,
    AMediaCodec_releaseOutputBuffer, AMediaFormat, AMediaFormat_new,
    AMediaFormat_setString, AMediaFormat_setInt32, AMediaFormat_delete,
};

pub struct NdkMediaCodecEncoder {
    codec: *mut AMediaCodec,
    width: u32,
    height: u32,
    rate: FrameRate,
    sps_pps: Option<Vec<u8>>,
}

impl NdkMediaCodecEncoder {
    pub fn new_h264(width: u32, height: u32, rate: FrameRate, bitrate_kbps: u32) -> Result<Self, ExportError> {
        unsafe {
            let mime = std::ffi::CString::new("video/avc").unwrap();
            let codec = AMediaCodec_createEncoderByType(mime.as_ptr());
            if codec.is_null() {
                return Err(ExportError::Encode("Hardware H.264 encoder unavailable".into()));
            }

            let format = AMediaFormat_new();
            let key_mime = std::ffi::CString::new("mime").unwrap();
            AMediaFormat_setString(format, key_mime.as_ptr(), mime.as_ptr());
            
            // Configure width, height, bitrate, framerate, I-frame interval
            AMediaFormat_setInt32(format, c"width".as_ptr(), width as i32);
            AMediaFormat_setInt32(format, c"height".as_ptr(), height as i32);
            AMediaFormat_setInt32(format, c"bitrate".as_ptr(), (bitrate_kbps * 1000) as i32);
            AMediaFormat_setInt32(format, c"frame-rate".as_ptr(), rate.as_f64().round() as i32);
            AMediaFormat_setInt32(format, c"i-frame-interval".as_ptr(), 2); // 2 second keyframes
            AMediaFormat_setInt32(format, c"color-format".as_ptr(), 21); // COLOR_FormatYUV420SemiPlanar (NV12)

            let configure_err = AMediaCodec_configure(codec, format, std::ptr::null_mut(), std::ptr::null_mut(), 1);
            AMediaFormat_delete(format);
            if configure_err != 0 {
                return Err(ExportError::Encode("AMediaCodec_configure failed".into()));
            }

            AMediaCodec_start(codec);
            Ok(Self { codec, width, height, rate, sps_pps: None })
        }
    }
}
```

### 2.2 Feed & Packet Extraction Loop
1. For each rendered frame, convert RGBA8 to NV12 (YUV420SP).
2. Call `AMediaCodec_dequeueInputBuffer(codec, timeout_us)` -> copy pixels -> `AMediaCodec_queueInputBuffer`.
3. Loop over `AMediaCodec_dequeueOutputBuffer(codec, &info, timeout_us)`:
   * If flag contains `BUFFER_FLAG_CODEC_CONFIG`, save SPS/PPS for MP4 `AvcConfig`.
   * Extract Annex-B NAL units, convert 4-byte start codes to Length-Prefixed MP4 packets (`filmcraft_isobmff::WriteSample`).
   * Pass packets to the existing `filmcraft_isobmff::Mp4Writer`.

---

## 3. Hardware Video Decoding for Media Import

Footage clips (MP4/MOV files imported by the user into the timeline) must decode rapidly without hogging the CPU:
* `AMediaExtractor` extracts video NAL units from storage.
* `AMediaCodec` runs in asynchronous mode on a background thread.
* Output buffers are rendered into a hardware texture (`SurfaceTexture` / Vulkan external image), which `wgpu` binds as an external texture.
* Zero copy CPU readback during timeline playback!

---

## 4. Audio Synchronization Across Segments

* All layer and keyframe timing in EffectCraft is governed by exact integer `Tick` (254,016,000,000 ticks/sec).
* Audio mixdown in [crates/render/src/audio.rs](file:///E:/effectcraft-mainformobile/crates/render/src/audio.rs) evaluates blocks with pre-roll, guaranteeing identical audio samples regardless of block sizing.
* During hardware MP4 export, AAC frames (1024 samples at 48,000 Hz = 21.33 ms) are synchronized with video Presentation Time Stamps (PTS).

---

## 5. Resumable Export & Crash Recovery

If export is cancelled by the user, interrupted by an incoming phone call, or killed by Android OS under thermal stress:

### 5.1 Checkpoint Journal File (`.ecrender`)
Before writing, the exporter creates a sidecar file next to the output:
```json
{
  "project_hash": "a1b2c3d4",
  "comp_id": 1,
  "total_frames": 300,
  "completed_frames": 184,
  "chunk_size": 30,
  "chunks": [
    {"index": 0, "path": "chunk_0.mp4", "frames": 30},
    {"index": 1, "path": "chunk_1.mp4", "frames": 30}
  ]
}
```

### 5.2 Chunk-Based Rendering
1. The exporter renders in chunks of 60–120 frames (2–4 seconds of video).
2. Each chunk is encoded via hardware `AMediaCodec` into an uncompressed fragment or temporary MP4 slice.
3. If an export is interrupted, the app re-opens the journal and prompts: *"Resume export from 60%?"*
4. Upon completing the final chunk, an MP4 box concatenator rapidly stitches the chunks into the final `video.mp4` without re-encoding frames.
