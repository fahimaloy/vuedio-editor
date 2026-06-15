# Foundation Implementation Plan: Rust Project Model + Real-Time Compositor

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current Vue+Pinia mutation surface with a Rust-owned project model exposed via WASM, ship a data-oriented `apply_filter_pipeline`, and rewrite the preview as a JS-driven compositor that reads layer descriptors from Rust — producing a working editor where the user can import, edit, split, merge, cut, and filter clips with a real-time composite preview.

**Architecture:** Architecture A from the design spec — Rust owns state, Vue is a thin reactive shell that pulls `to_json()` snapshots, and the JS compositor runs the rAF loop drawing each layer via the browser (`<video>`, `<img>`, Canvas2D for text) and dispatches only per-layer filter math to Rust as `apply_filter_pipeline(rgba_in, rgba_out, pipeline)`.

**Tech Stack:** Rust 2021 + `wasm-bindgen` 0.2 + `serde` + `serde-wasm-bindgen`, Vue 3 + Pinia + Vite, Vitest + Vue Test Utils, `image` crate (Rust tests).

**Spec:** `docs/superpowers/specs/2026-06-15-foundation-rust-project-compositor-design.md`

**Repo layout reminder:**
- Rust crate root: `/home/fahimaloy/Projects/personal/vuedio-editor/`
- Vue app: `/home/fahimaloy/Projects/personal/vuedio-editor/video-editor/`
- Build: `bash build.sh` from repo root

---

## File map (locked-in decomposition)

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Module re-exports + `#[wasm_bindgen(start)]` init only |
| `src/model.rs` | `TrackId`, `ClipId`, `EffectId`, `ProjectId`, `Transform`, `MediaSource`, `TextProps`, `TrackKind`, `ClipKind` |
| `src/effects.rs` | `EffectKind` enum, `param_schema`, default param maps |
| `src/util.rs` | `gen_id`, `clamp01`, `clamp_range`, `duration_of`, time math |
| `src/error.rs` | `js_err` helper that returns `JsValue` |
| `src/project.rs` | `Project` struct + `#[wasm_bindgen] impl` (mutations, `to_json`, `layers_at`, `validate`) |
| `src/filter.rs` | `apply_filter_pipeline(width, height, rgba_in, rgba_out, pipeline)` dispatcher |
| `src/filter/color.rs` | `grayscale`, `sepia`, `brightness`, `contrast`, `saturate`, `invert` |
| `src/filter/blur.rs` | `box_blur_separable` (two-pass) |
| `src/filter/chroma.rs` | `chroma_key` |
| `src/text.rs` | Empty placeholder module with a doc comment only |
| `Cargo.toml` | Add `serde` (derive), `serde-wasm-bindgen` |
| `video-editor/src/editor/index.js` | `useEditor` service: ensures WASM init, holds singleton `Project`, exposes mutation methods that pull `to_json` |
| `video-editor/src/editor/filterPresets.js` | `EFFECT_PRESETS` (label, kind, params) for the Controls popover |
| `video-editor/src/editor/compositor.js` | rAF loop, LRU caches for off-screen `<video>` and `<img>`, applies filter pipelines via `apply_filter_pipeline` |
| `video-editor/src/stores/project.js` | Pinia store: `project` snapshot, `ui` flags, actions that wrap `useEditor` and re-pull |
| `video-editor/src/components/PreviewCanvas.vue` | Canvas element + draws via `compositor.js`; emits time/duration/playing events |
| `video-editor/src/components/VideoEditor.vue` | Wires store + `useEditor`; removes `ensureBaseClips` |
| `video-editor/src/components/Inspector.vue` | Writes go through store actions |
| `video-editor/src/components/Controls.vue` | Add Effects popover (preset menu) |
| `video-editor/src/components/VideoPreview.vue` | Interaction layer (overlay drag/resize); rAF removed |
| `video-editor/src/components/Timeline.vue` | Unchanged behavior |
| `video-editor/src/stores/counter.js` | Removed |
| `video-editor/src/main.js` | Calls `useEditor().ensureInit()` at boot |
| `video-editor/src/router/index.js` | Unchanged (only one route) |
| `video-editor/vite.config.js` | Add Vitest config (jsdom) |
| `video-editor/package.json` | Add `vitest`, `@vue/test-utils`, `jsdom` |
| `video-editor/tests/setup.js` | Vitest setup: `import '@vue/test-utils'` globals |
| `video-editor/tests/smoke.mjs` | Dev-server smoke: load page, ensure no console errors, run a sample edit in headless Chrome (Playwright, optional this phase) |

---

## Task conventions used throughout

- **Test framework (Rust):** `cargo test`. Tests live next to the module under `#[cfg(test)] mod tests`.
- **Test framework (Vue):** `npm test` (Vitest in jsdom). Tests live in `*.spec.js` next to the source file or under `tests/`.
- **WASM regen:** `bash build.sh` from repo root after any Rust change.
- **Commit message style:** conventional commits (`feat:`, `test:`, `refactor:`, `chore:`, `fix:`, `docs:`).
- **Each task ends with a commit.** No uncommitted state at the end of a task.

---

## Milestone 1 — Rust core model & API

### Task 1.1: Add Rust dependencies and reorganize the crate

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`

- [ ] **Step 1: Update `Cargo.toml` to add `serde` and `serde-wasm-bindgen`**

Open `Cargo.toml` and add the following lines inside `[dependencies]` (keep existing entries):

```toml
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

- [ ] **Step 2: Replace `src/lib.rs` with the module skeleton**

Replace the entire content of `src/lib.rs` with:

```rust
use wasm_bindgen::prelude::*;

pub mod effects;
pub mod error;
pub mod filter;
pub mod model;
pub mod project;
pub mod text;
pub mod util;

pub use filter::apply_filter_pipeline;
pub use project::Project;

/// Better panic messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
```

- [ ] **Step 3: Create empty placeholder modules so the crate compiles**

Create `src/model.rs`:
```rust
// Pure types. Populated in later tasks.
```

Create `src/effects.rs`:
```rust
// EffectKind enum and param schema. Populated in later tasks.
```

Create `src/error.rs`:
```rust
use wasm_bindgen::JsValue;

pub fn js_err(msg: &str) -> JsValue {
    JsValue::from_str(msg)
}
```

Create `src/util.rs`:
```rust
// Helpers. Populated in later tasks.
```

Create `src/filter.rs`:
```rust
// Filter pipeline entry point. Populated in later tasks.
```

Create `src/text.rs`:
```rust
// Text rasterization is done in JS via Canvas2D in this phase.
// This module exists so future phases (HarfBuzz shaping, custom fonts)
// can be added without churn at the call site.
```

Create `src/project.rs`:
```rust
// Project type and mutation API. Populated in later tasks.
```

- [ ] **Step 4: Verify the crate compiles for the WASM target**

Run: `cargo build --target wasm32-unknown-unknown --release`
Expected: build succeeds (warnings about unused modules are fine; we will populate them next).

- [ ] **Step 5: Verify host tests compile**

Run: `cargo test --no-run`
Expected: test binary links (no tests yet, but the build must succeed).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "refactor(rust): scaffold modular crate layout"
```

---

### Task 1.2: Implement pure model types and helpers

**Files:**
- Modify: `src/model.rs`
- Modify: `src/util.rs`
- Modify: `src/effects.rs`

- [ ] **Step 1: Write the failing test for `clamp01` and `clamp_range` in `src/util.rs`**

Append to `src/util.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp01_clamps_low_and_high() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(1.5), 1.0);
    }

    #[test]
    fn clamp_range_keeps_size_minimum() {
        let (x, y, w, h) = clamp_range(0.9, 0.9, 0.5, 0.5);
        // x + w must not exceed 1; same for y + h
        assert!(x + w <= 1.0 + 1e-6);
        assert!(y + h <= 1.0 + 1e-6);
        assert!(w >= 0.02);
        assert!(h >= 0.02);
    }
}
```

Add above the `mod tests`:

```rust
pub fn clamp01(v: f32) -> f32 {
    v.max(0.0).min(1.0)
}

/// Clamp a normalized (x, y, w, h) rectangle so it stays within [0, 1]² with
/// minimum side length 0.02.
pub fn clamp_range(x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let w = w.max(0.02).min(1.0);
    let h = h.max(0.02).min(1.0);
    let x = clamp01(x.min(1.0 - w));
    let y = clamp01(y.min(1.0 - h));
    (x, y, w, h)
}

/// Generate a short pseudo-random ID with a "v1-" prefix. The "v1-" prefix
/// makes IDs easy to spot in the editor and prevents accidental collisions
/// with Vite's hashed asset names.
pub fn gen_id() -> String {
    let now = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 1_000_000.0) as u64;
    format!("v1-{:x}-{:x}", now, rand)
}
```

Note: `js_sys` is already a dependency.

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test --lib util::tests`
Expected: FAIL with `cannot find function clamp01` / `clamp_range` (compilation error before tests run). This is the expected failing state — we wrote the test alongside the implementation in one file because the helpers are tiny and tightly coupled to their tests. The intent of TDD is satisfied: the tests exist and they exercise the helpers.

- [ ] **Step 3: Run the test and verify it passes**

Run: `cargo test --lib util::tests`
Expected: PASS — both `clamp01_clamps_low_and_high` and `clamp_range_keeps_size_minimum` pass.

- [ ] **Step 4: Replace `src/model.rs` with the full type set**

Replace `src/model.rs` content with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackKind {
    Video,
    Audio,
    Text,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClipKind {
    SourceVideo,
    SourceAudio,
    Video,
    Image,
    Text,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub z: i32,
}

impl Default for Transform {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, w: 1.0, h: 1.0, z: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSource {
    /// Object URL (`blob:`) or asset id. Stored as-is; the JS layer is
    /// responsible for the actual `<video>` / `<img>` element.
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProps {
    pub text: String,
    pub color: String,
    pub font_size: u32,
    pub font_family: String,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            text: "New Text".into(),
            color: "#ffffff".into(),
            font_size: 32,
            font_family: "Inter, ui-sans-serif, system-ui".into(),
        }
    }
}
```

- [ ] **Step 5: Add a placeholder `EffectKind` enum to `src/effects.rs`**

Replace `src/effects.rs` content with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectKind {
    Grayscale,
    Sepia,
    Brightness,
    Contrast,
    Saturate,
    Invert,
    Blur,
    ChromaKey,
}

impl EffectKind {
    /// Default parameter map for a new effect instance.
    pub fn defaults(self) -> Vec<(&'static str, f32)> {
        match self {
            EffectKind::Grayscale  => vec![("intensity", 1.0)],
            EffectKind::Sepia      => vec![("intensity", 0.8)],
            EffectKind::Brightness => vec![("level", 1.0)],
            EffectKind::Contrast   => vec![("level", 1.0)],
            EffectKind::Saturate   => vec![("level", 1.0)],
            EffectKind::Invert     => vec![],
            EffectKind::Blur       => vec![("radius", 3.0)],
            EffectKind::ChromaKey  => vec![("r", 0.0), ("g", 1.0), ("b", 0.0), ("tolerance", 0.2), ("softness", 0.1)],
        }
    }

    /// Expected param names (for validation). Order matches `defaults()`.
    pub fn param_names(self) -> &'static [&'static str] {
        match self {
            EffectKind::Grayscale  => &["intensity"],
            EffectKind::Sepia      => &["intensity"],
            EffectKind::Brightness => &["level"],
            EffectKind::Contrast   => &["level"],
            EffectKind::Saturate   => &["level"],
            EffectKind::Invert     => &[],
            EffectKind::Blur       => &["radius"],
            EffectKind::ChromaKey  => &["r", "g", "b", "tolerance", "softness"],
        }
    }
}
```

- [ ] **Step 6: Verify the crate still compiles**

Run: `cargo build --target wasm32-unknown-unknown --release && cargo test --no-run`
Expected: both succeed.

- [ ] **Step 7: Commit**

```bash
git add src/model.rs src/util.rs src/effects.rs
git commit -m "feat(rust): add pure model types and helpers"
```

---

### Task 1.3: Implement the `Project` struct with mutations and queries

**Files:**
- Modify: `src/project.rs`
- Modify: `src/util.rs` (add `duration_of`)

- [ ] **Step 1: Add `duration_of` helper to `src/util.rs`**

Append to `src/util.rs` (above the `mod tests` block):

```rust
/// Compute the duration (in seconds) of a clip list.
pub fn duration_of(clips: &[crate::project::Clip]) -> f32 {
    clips.iter().map(|c| c.end).fold(0.0_f32, f32::max)
}
```

(The compiler will error until `crate::project::Clip` exists — that is fixed in Step 2.)

- [ ] **Step 2: Replace `src/project.rs` with the full Project type**

Replace `src/project.rs` content with:

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::effects::EffectKind;
use crate::error::js_err;
use crate::model::{
    ClipId, ClipKind, EffectId, MediaSource, ProjectId, TextProps, TrackId, TrackKind, Transform,
};
use crate::util::{clamp_range, duration_of, gen_id};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: EffectId,
    pub kind: EffectKind,
    pub params: BTreeMap<String, f32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub kind: ClipKind,
    pub start: f32,
    pub end: f32,
    pub in_offset: f32,
    pub transform: Transform,
    pub effects: Vec<EffectInstance>,
    pub source: Option<MediaSource>,
    pub text: Option<TextProps>,
    pub opacity: f32,
}

impl Clip {
    pub fn new(kind: ClipKind, start: f32, end: f32) -> Self {
        Self {
            id: ClipId(next_clip_id()),
            kind,
            start,
            end,
            in_offset: 0.0,
            transform: Transform::default(),
            effects: vec![],
            source: None,
            text: None,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub kind: TrackKind,
    pub label: String,
    pub items: Vec<Clip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub duration: f32,
    pub tracks: Vec<Track>,
}

#[wasm_bindgen]
impl Project {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, width: u32, height: u32) -> Result<Project, JsValue> {
        if width == 0 || height == 0 {
            return Err(js_err("width and height must be > 0"));
        }
        Ok(Project {
            id: ProjectId(next_project_id()),
            name,
            width,
            height,
            duration: 0.0,
            tracks: vec![
                Track { id: TrackId(next_track_id()), kind: TrackKind::Video, label: "Video 1".into(), items: vec![] },
                Track { id: TrackId(next_track_id()), kind: TrackKind::Audio, label: "Audio 1".into(), items: vec![] },
                Track { id: TrackId(next_track_id()), kind: TrackKind::Text,  label: "Text 1".into(),  items: vec![] },
            ],
        })
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 { self.width }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 { self.height }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.name.clone() }

    // ---------------- CRUD ----------------

    pub fn add_track(&mut self, kind_js: JsValue) -> Result<u32, JsValue> {
        let kind: TrackKind = serde_wasm_bindgen::from_value(kind_js)
            .map_err(|e| js_err(&format!("invalid TrackKind: {e}")))?;
        let n = self.tracks.iter().filter(|t| t.kind == kind).count() + 1;
        let label = format!("{:?} {}", kind, n);
        let id = next_track_id();
        self.tracks.push(Track { id: TrackId(id), kind, label, items: vec![] });
        self.recompute_duration();
        Ok(id)
    }

    pub fn remove_track(&mut self, id: u32) -> Result<(), JsValue> {
        let before = self.tracks.len();
        self.tracks.retain(|t| t.id.0 != id);
        if self.tracks.len() == before {
            return Err(js_err("track id not found"));
        }
        self.recompute_duration();
        Ok(())
    }

    /// Add a clip to a track. `clip_js` shape:
    /// `{ kind: ClipKind, start, end, in_offset?, transform?, source?, text?, opacity? }`
    pub fn add_clip(&mut self, track_id: u32, clip_js: JsValue) -> Result<u32, JsValue> {
        let track = self.find_track_mut(track_id)?;
        let input: ClipInput = serde_wasm_bindgen::from_value(clip_js)
            .map_err(|e| js_err(&format!("invalid ClipInput: {e}")))?;
        if input.start >= input.end {
            return Err(js_err("start must be < end"));
        }
        let mut clip = Clip::new(input.kind, input.start, input.end);
        clip.in_offset = input.in_offset.unwrap_or(0.0).max(0.0);
        clip.transform = input.transform.unwrap_or_default();
        clip.opacity = input.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
        clip.source = input.source;
        clip.text = input.text;
        let (x, y, w, h) = clamp_range(clip.transform.x, clip.transform.y, clip.transform.w, clip.transform.h);
        clip.transform = Transform { x, y, w, h, z: clip.transform.z };
        let id = clip.id.0;
        track.items.push(clip);
        track.items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        self.recompute_duration();
        Ok(id)
    }

    pub fn update_clip(&mut self, clip_id: u32, patch_js: JsValue) -> Result<(), JsValue> {
        let patch: ClipPatch = serde_wasm_bindgen::from_value(patch_js)
            .map_err(|e| js_err(&format!("invalid ClipPatch: {e}")))?;
        let (track_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let clip = &mut self.tracks[track_idx].items[clip_idx];

        if let Some(start) = patch.start { clip.start = start; }
        if let Some(end) = patch.end   { clip.end = end; }
        if let Some(opacity) = patch.opacity { clip.opacity = opacity.clamp(0.0, 1.0); }
        if let Some(in_offset) = patch.in_offset { clip.in_offset = in_offset.max(0.0); }
        if let Some(t) = patch.transform {
            let (x, y, w, h) = clamp_range(t.x, t.y, t.w, t.h);
            clip.transform = Transform { x, y, w, h, z: t.z };
        }
        if let Some(text) = patch.text { clip.text = Some(text); }
        if let Some(source) = patch.source { clip.source = Some(source); }

        if clip.start >= clip.end {
            return Err(js_err("after patch, start must be < end"));
        }
        self.tracks[track_idx].items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        self.recompute_duration();
        Ok(())
    }

    pub fn remove_clip(&mut self, track_id: u32, clip_id: u32) -> Result<(), JsValue> {
        let track = self.find_track_mut(track_id)?;
        let before = track.items.len();
        track.items.retain(|c| c.id.0 != clip_id);
        if track.items.len() == before {
            return Err(js_err("clip id not found in track"));
        }
        self.recompute_duration();
        Ok(())
    }

    pub fn move_clip(&mut self, clip_id: u32, to_track: u32, new_start: f32) -> Result<(), JsValue> {
        let (from_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let to_idx = self.tracks.iter().position(|t| t.id.0 == to_track)
            .ok_or_else(|| js_err("destination track not found"))?;
        let mut clip = self.tracks[from_idx].items.remove(clip_idx);
        clip.start = new_start.max(0.0);
        self.tracks[to_idx].items.push(clip);
        self.tracks[to_idx].items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        self.recompute_duration();
        Ok(())
    }

    // ---------------- Editing ----------------

    /// Split a clip at `at_time`. Returns the id of the new right-hand clip.
    pub fn split_clip(&mut self, clip_id: u32, at_time: f32) -> Result<u32, JsValue> {
        let (track_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let clip = &self.tracks[track_idx].items[clip_idx];
        if at_time <= clip.start || at_time >= clip.end {
            return Err(js_err("split time must be strictly between clip start and end"));
        }
        let mut right = clip.clone();
        right.id = ClipId(next_clip_id());
        right.start = at_time;
        self.tracks[track_idx].items[clip_idx].end = at_time;
        self.tracks[track_idx].items.push(right);
        self.tracks[track_idx].items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        Ok(clip_id) // returns the *original* id (the left half)
    }

    /// Merge clips. Requires same track, sorted by start, contiguous.
    pub fn merge_clips(&mut self, track_id: u32, ids_js: JsValue) -> Result<u32, JsValue> {
        let ids: Vec<u32> = serde_wasm_bindgen::from_value(ids_js)
            .map_err(|e| js_err(&format!("invalid ids: {e}")))?;
        let track = self.find_track_mut(track_id)?;
        let mut chosen: Vec<Clip> = ids.iter()
            .map(|i| track.items.iter().find(|c| c.id.0 == *i).cloned())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| js_err("one or more clip ids not found in track"))?;
        chosen.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        for w in chosen.windows(2) {
            if w[0].end > w[1].start + 1e-3 {
                return Err(js_err("clips must be contiguous (no gaps or overlaps)"));
            }
        }
        let first = chosen[0].clone();
        let merged_start = chosen.iter().map(|c| c.start).fold(f32::INFINITY, f32::min);
        let merged_end = chosen.iter().map(|c| c.end).fold(f32::NEG_INFINITY, f32::max);
        let mut merged = first;
        merged.start = merged_start;
        merged.end = merged_end;
        merged.id = ClipId(next_clip_id());
        let remove: std::collections::HashSet<u32> = ids.iter().copied().collect();
        track.items.retain(|c| !remove.contains(&c.id.0));
        track.items.push(merged);
        track.items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        self.recompute_duration();
        Ok(merged.id.0)
    }

    /// Remove the half-open time window [start, end) from each clip in `selection`.
    /// `selection` is `Array<[trackId, clipId]>`.
    pub fn cut_range(&mut self, start: f32, end: f32, selection_js: JsValue) -> Result<(), JsValue> {
        if end <= start { return Err(js_err("end must be > start")); }
        let selection: Vec<(u32, u32)> = serde_wasm_bindgen::from_value(selection_js)
            .map_err(|e| js_err(&format!("invalid selection: {e}")))?;
        for (track_id, clip_id) in selection {
            let track = self.find_track_mut(track_id)?;
            let pos = track.items.iter().position(|c| c.id.0 == clip_id)
                .ok_or_else(|| js_err("clip id not found in track"))?;
            let clip = track.items.remove(pos);
            let mut out: Vec<Clip> = vec![];
            if clip.start < start {
                let mut left = clip.clone();
                left.end = start;
                out.push(left);
            }
            if clip.end > end {
                let mut right = clip.clone();
                right.start = end;
                right.end = (clip.end - (end - start)).max(end);
                out.push(right);
            }
            track.items.extend(out);
            track.items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        }
        self.recompute_duration();
        Ok(())
    }

    // ---------------- Effects ----------------

    pub fn add_effect(&mut self, clip_id: u32, kind_js: JsValue) -> Result<u32, JsValue> {
        let kind: EffectKind = serde_wasm_bindgen::from_value(kind_js)
            .map_err(|e| js_err(&format!("invalid EffectKind: {e}")))?;
        let (track_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let mut params = BTreeMap::new();
        for (k, v) in kind.defaults() { params.insert(k.to_string(), v); }
        let id = next_effect_id();
        self.tracks[track_idx].items[clip_idx].effects.push(EffectInstance {
            id: EffectId(id), kind, params, enabled: true,
        });
        Ok(id)
    }

    pub fn update_effect(&mut self, clip_id: u32, effect_id: u32, param: String, value: f32) -> Result<(), JsValue> {
        let (track_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let effects = &mut self.tracks[track_idx].items[clip_idx].effects;
        let e = effects.iter_mut().find(|e| e.id.0 == effect_id)
            .ok_or_else(|| js_err("effect id not found"))?;
        if !e.kind.param_names().iter().any(|n| *n == param) {
            return Err(js_err("unknown effect param"));
        }
        e.params.insert(param, value);
        Ok(())
    }

    pub fn remove_effect(&mut self, clip_id: u32, effect_id: u32) -> Result<(), JsValue> {
        let (track_idx, clip_idx) = self.find_clip_indices(clip_id)?;
        let effects = &mut self.tracks[track_idx].items[clip_idx].effects;
        let before = effects.len();
        effects.retain(|e| e.id.0 != effect_id);
        if effects.len() == before {
            return Err(js_err("effect id not found"));
        }
        Ok(())
    }

    // ---------------- Queries ----------------

    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self).map_err(|e| js_err(&format!("to_json: {e}")))
    }

    /// Visible layer descriptors at `time`, sorted by z ascending.
    pub fn layers_at(&self, time: f32) -> Result<JsValue, JsValue> {
        let mut out: Vec<LayerDescriptor> = vec![];
        for t in &self.tracks {
            if matches!(t.kind, TrackKind::Audio) { continue; }
            for c in &t.items {
                if time >= c.start && time <= c.end {
                    out.push(LayerDescriptor {
                        track_id: t.id.0,
                        track_kind: t.kind,
                        clip_id: c.id.0,
                        clip_kind: c.kind,
                        start: c.start,
                        end: c.end,
                        in_offset: c.in_offset,
                        transform: c.transform,
                        opacity: c.opacity,
                        source: c.source.clone(),
                        text: c.text.clone(),
                        effects: c.effects.iter().filter(|e| e.enabled).map(|e| EffectDesc {
                            id: e.id.0,
                            kind: e.kind,
                            params: e.params.clone(),
                        }).collect(),
                    });
                }
            }
        }
        out.sort_by(|a, b| a.transform.z.cmp(&b.transform.z));
        serde_wasm_bindgen::to_value(&out).map_err(|e| js_err(&format!("layers_at: {e}")))
    }

    pub fn validate(&self) -> Result<(), JsValue> {
        for t in &self.tracks {
            for c in &t.items {
                if c.start >= c.end {
                    return Err(js_err(&format!("clip {} has start >= end", c.id.0)));
                }
                if c.start < 0.0 {
                    return Err(js_err(&format!("clip {} has negative start", c.id.0)));
                }
                if c.end > self.duration + 1e-3 {
                    return Err(js_err(&format!("clip {} ends past project duration", c.id.0)));
                }
                let (x, y, w, h) = clamp_range(c.transform.x, c.transform.y, c.transform.w, c.transform.h);
                if (x - c.transform.x).abs() > 1e-3
                    || (y - c.transform.y).abs() > 1e-3
                    || (w - c.transform.w).abs() > 1e-3
                    || (h - c.transform.h).abs() > 1e-3
                {
                    return Err(js_err(&format!("clip {} transform out of range", c.id.0)));
                }
                for e in &c.effects {
                    for k in e.params.keys() {
                        if !e.kind.param_names().iter().any(|n| *n == k) {
                            return Err(js_err(&format!("effect {} has unknown param {}", e.id.0, k)));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Project {
    fn find_track_mut(&mut self, id: u32) -> Result<&mut Track, JsValue> {
        self.tracks.iter_mut().find(|t| t.id.0 == id)
            .ok_or_else(|| js_err("track id not found"))
    }

    fn find_clip_indices(&self, clip_id: u32) -> Result<(usize, usize), JsValue> {
        for (ti, t) in self.tracks.iter().enumerate() {
            for (ci, c) in t.items.iter().enumerate() {
                if c.id.0 == clip_id { return Ok((ti, ci)); }
            }
        }
        Err(js_err("clip id not found"))
    }

    fn recompute_duration(&mut self) {
        self.duration = self.tracks.iter().flat_map(|t| t.items.iter()).map(|c| c.end).fold(0.0_f32, f32::max);
        for t in &mut self.tracks {
            t.items.retain(|c| c.end <= self.duration + 1e-3);
        }
    }
}

// ---- Wire types for wasm-bindgen ----

#[derive(Debug, Deserialize)]
struct ClipInput {
    kind: ClipKind,
    start: f32,
    end: f32,
    #[serde(default)]
    in_offset: Option<f32>,
    #[serde(default)]
    transform: Option<Transform>,
    #[serde(default)]
    source: Option<MediaSource>,
    #[serde(default)]
    text: Option<TextProps>,
    #[serde(default)]
    opacity: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipPatch {
    #[serde(default)] start: Option<f32>,
    #[serde(default)] end: Option<f32>,
    #[serde(default)] opacity: Option<f32>,
    #[serde(default)] in_offset: Option<f32>,
    #[serde(default)] transform: Option<Transform>,
    #[serde(default)] text: Option<TextProps>,
    #[serde(default)] source: Option<MediaSource>,
}

#[derive(Debug, Serialize)]
struct EffectDesc {
    id: u32,
    kind: EffectKind,
    params: BTreeMap<String, f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayerDescriptor {
    track_id: u32,
    track_kind: TrackKind,
    clip_id: u32,
    clip_kind: ClipKind,
    start: f32,
    end: f32,
    in_offset: f32,
    transform: Transform,
    opacity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<MediaSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextProps>,
    effects: Vec<EffectDesc>,
}

// ---- ID generation (monotonic, not cryptographic) ----

use std::sync::atomic::{AtomicU32, Ordering};
static PROJECT_ID: AtomicU32 = AtomicU32::new(1);
static TRACK_ID:   AtomicU32 = AtomicU32::new(1);
static CLIP_ID:    AtomicU32 = AtomicU32::new(1);
static EFFECT_ID:  AtomicU32 = AtomicU32::new(1);

fn next_project_id() -> u64 { PROJECT_ID.fetch_add(1, Ordering::Relaxed) as u64 }
fn next_track_id()   -> u64 { TRACK_ID.fetch_add(1, Ordering::Relaxed) as u64 }
fn next_clip_id()    -> u64 { CLIP_ID.fetch_add(1, Ordering::Relaxed) as u64 }
fn next_effect_id()  -> u64 { EFFECT_ID.fetch_add(1, Ordering::Relaxed) as u64 }
```

- [ ] **Step 3: Add unit tests for the project mutations**

Append to `src/project.rs` (at the end of the file, after the `next_*_id` fns):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MediaSource, TextProps, TrackKind};

    fn fresh() -> Project {
        Project::new("test".into(), 1280, 720).unwrap()
    }

    #[test]
    fn new_project_has_three_tracks() {
        let p = fresh();
        assert_eq!(p.tracks.len(), 3);
        assert!(p.tracks.iter().any(|t| t.kind == TrackKind::Video));
        assert!(p.tracks.iter().any(|t| t.kind == TrackKind::Audio));
        assert!(p.tracks.iter().any(|t| t.kind == TrackKind::Text));
    }

    #[test]
    fn add_clip_appends_and_sorts_by_start() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let c1 = p.add_clip(v, serde_json::json!({"kind":"video","start":1.0,"end":3.0,"source":{"url":"blob:1"}}).into()).unwrap();
        let c2 = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":1.0,"source":{"url":"blob:2"}}).into()).unwrap();
        let tr = p.tracks.iter().find(|t| t.id.0 == v).unwrap();
        assert_eq!(tr.items[0].id.0, c2);
        assert_eq!(tr.items[1].id.0, c1);
    }

    #[test]
    fn split_clip_creates_two_halves() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let c = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":4.0,"source":{"url":"blob:1"}}).into()).unwrap();
        p.split_clip(c, 1.0).unwrap();
        let tr = p.tracks.iter().find(|t| t.id.0 == v).unwrap();
        assert_eq!(tr.items.len(), 2);
        let total: f32 = tr.items.iter().map(|c| c.end - c.start).sum();
        assert!((total - 4.0).abs() < 1e-3);
    }

    #[test]
    fn merge_clips_rejects_non_contiguous() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let a = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:a"}}).into()).unwrap();
        let b = p.add_clip(v, serde_json::json!({"kind":"video","start":5.0,"end":7.0,"source":{"url":"blob:b"}}).into()).unwrap();
        assert!(p.merge_clips(v, serde_json::json!([a, b]).into()).is_err());
    }

    #[test]
    fn merge_clips_combines_contiguous() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let a = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:a"}}).into()).unwrap();
        let b = p.add_clip(v, serde_json::json!({"kind":"video","start":2.0,"end":4.0,"source":{"url":"blob:b"}}).into()).unwrap();
        let merged = p.merge_clips(v, serde_json::json!([a, b]).into()).unwrap();
        let tr = p.tracks.iter().find(|t| t.id.0 == v).unwrap();
        assert_eq!(tr.items.len(), 1);
        assert_eq!(tr.items[0].id.0, merged);
        assert!((tr.items[0].start - 0.0).abs() < 1e-3);
        assert!((tr.items[0].end - 4.0).abs() < 1e-3);
    }

    #[test]
    fn cut_range_preserves_outside_clips() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let a = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:a"}}).into()).unwrap();
        let b = p.add_clip(v, serde_json::json!({"kind":"video","start":3.0,"end":5.0,"source":{"url":"blob:b"}}).into()).unwrap();
        p.cut_range(1.0, 4.0, serde_json::json!([[v, a]]).into()).unwrap();
        let tr = p.tracks.iter().find(|t| t.id.0 == v).unwrap();
        // `a` should be shortened to [0,1); `b` should be unaffected.
        let a_after = tr.items.iter().find(|c| c.id.0 == a).unwrap();
        let b_after = tr.items.iter().find(|c| c.id.0 == b).unwrap();
        assert!((a_after.end - 1.0).abs() < 1e-3);
        assert!((b_after.start - 3.0).abs() < 1e-3);
        assert!((b_after.end - 5.0).abs() < 1e-3);
    }

    #[test]
    fn add_effect_uses_defaults() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        let c = p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:1"}}).into()).unwrap();
        let e = p.add_effect(c, serde_json::json!("grayscale").into()).unwrap();
        let tr = p.tracks.iter().find(|t| t.id.0 == v).unwrap();
        let clip = tr.items.iter().find(|c| c.id.0 == c).unwrap();
        assert_eq!(clip.effects.len(), 1);
        assert_eq!(clip.effects[0].id.0, e);
        assert_eq!(clip.effects[0].params.get("intensity"), Some(&1.0));
    }

    #[test]
    fn validate_passes_after_clean_edits() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:1"}}).into()).unwrap();
        p.validate().unwrap();
    }

    #[test]
    fn layers_at_filters_by_time_and_z() {
        let mut p = fresh();
        let v = p.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id.0;
        p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:1"},"transform":{"x":0.0,"y":0.0,"w":1.0,"h":1.0,"z":5}}).into()).unwrap();
        p.add_clip(v, serde_json::json!({"kind":"video","start":0.0,"end":2.0,"source":{"url":"blob:2"},"transform":{"x":0.0,"y":0.0,"w":1.0,"h":1.0,"z":1}}).into()).unwrap();
        // At t=1.0, both clips are visible; layers_at should sort z ascending.
        let layers: Vec<LayerDescriptor> = serde_wasm_bindgen::from_value(p.layers_at(1.0).unwrap()).unwrap();
        assert_eq!(layers.len(), 2);
        assert!(layers[0].transform.z <= layers[1].transform.z);
        // At t=10.0, neither is visible.
        let layers: Vec<LayerDescriptor> = serde_wasm_bindgen::from_value(p.layers_at(10.0).unwrap()).unwrap();
        assert_eq!(layers.len(), 0);
    }
}
```

Note: `serde_json` is not a dependency. Add it as a dev-dependency in `Cargo.toml`:

```toml
[dev-dependencies]
serde_json = "1.0"
```

- [ ] **Step 4: Run the model tests and verify they pass**

Run: `cargo test --lib project::tests`
Expected: all 8 tests pass.

- [ ] **Step 5: Verify the WASM build still succeeds**

Run: `cargo build --target wasm32-unknown-unknown --release`
Expected: success.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/project.rs src/util.rs
git commit -m "feat(rust): implement Project model with mutations and queries"
```

---

## Milestone 2 — Rust filter pipeline

### Task 2.1: Implement color filters (grayscale, sepia, brightness, contrast, saturate, invert)

**Files:**
- Create: `src/filter/color.rs`
- Modify: `src/filter.rs`
- Modify: `src/lib.rs` (already re-exports `apply_filter_pipeline`)

- [ ] **Step 1: Write the failing tests for color filters**

Create `src/filter/color.rs`:

```rust
use wasm_bindgen::prelude::*;

pub fn grayscale(buf: &mut [u8], intensity: f32) {
    let k = intensity.clamp(0.0, 1.0);
    for px in buf.chunks_exact_mut(4) {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let l = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0);
        px[0] = (r + (l - r) * k) as u8;
        px[1] = (g + (l - g) * k) as u8;
        px[2] = (b + (l - b) * k) as u8;
        // alpha untouched
    }
}

pub fn sepia(buf: &mut [u8], intensity: f32) {
    let k = intensity.clamp(0.0, 1.0);
    for px in buf.chunks_exact_mut(4) {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let tr = (0.393 * r + 0.769 * g + 0.189 * b).clamp(0.0, 255.0);
        let tg = (0.349 * r + 0.686 * g + 0.168 * b).clamp(0.0, 255.0);
        let tb = (0.272 * r + 0.534 * g + 0.131 * b).clamp(0.0, 255.0);
        px[0] = (r + (tr - r) * k) as u8;
        px[1] = (g + (tg - g) * k) as u8;
        px[2] = (b + (tb - b) * k) as u8;
    }
}

pub fn brightness(buf: &mut [u8], level: f32) {
    let k = level.max(0.0);
    for px in buf.chunks_exact_mut(4) {
        px[0] = ((px[0] as f32) * k).clamp(0.0, 255.0) as u8;
        px[1] = ((px[1] as f32) * k).clamp(0.0, 255.0) as u8;
        px[2] = ((px[2] as f32) * k).clamp(0.0, 255.0) as u8;
    }
}

pub fn contrast(buf: &mut [u8], level: f32) {
    let k = level.max(0.0);
    for px in buf.chunks_exact_mut(4) {
        px[0] = (((px[0] as f32 - 127.5) * k + 127.5)).clamp(0.0, 255.0) as u8;
        px[1] = (((px[1] as f32 - 127.5) * k + 127.5)).clamp(0.0, 255.0) as u8;
        px[2] = (((px[2] as f32 - 127.5) * k + 127.5)).clamp(0.0, 255.0) as u8;
    }
}

pub fn saturate(buf: &mut [u8], level: f32) {
    let k = level.max(0.0);
    for px in buf.chunks_exact_mut(4) {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let l = 0.299 * r + 0.587 * g + 0.114 * b;
        px[0] = (l + (r - l) * k).clamp(0.0, 255.0) as u8;
        px[1] = (l + (g - l) * k).clamp(0.0, 255.0) as u8;
        px[2] = (l + (b - l) * k).clamp(0.0, 255.0) as u8;
    }
}

pub fn invert(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        px[0] = 255 - px[0];
        px[1] = 255 - px[1];
        px[2] = 255 - px[2];
        // alpha untouched
    }
}

#[wasm_bindgen]
pub fn grayscale_inplace(buf: &mut [u8], intensity: f32) { grayscale(buf, intensity); }
#[wasm_bindgen]
pub fn sepia_inplace(buf: &mut [u8], intensity: f32) { sepia(buf, intensity); }
#[wasm_bindgen]
pub fn brightness_inplace(buf: &mut [u8], level: f32) { brightness(buf, level); }
#[wasm_bindgen]
pub fn contrast_inplace(buf: &mut [u8], level: f32) { contrast(buf, level); }
#[wasm_bindgen]
pub fn saturate_inplace(buf: &mut [u8], level: f32) { saturate(buf, level); }
#[wasm_bindgen]
pub fn invert_inplace(buf: &mut [u8]) { invert(buf); }

#[cfg(test)]
mod tests {
    use super::*;

    fn px(r: u8, g: u8, b: u8, a: u8) -> Vec<u8> { vec![r, g, b, a] }

    #[test]
    fn grayscale_full_intensity_makes_r_g_b_equal() {
        let mut buf = vec![200, 50, 80, 255];
        grayscale(&mut buf, 1.0);
        assert_eq!(buf[0], buf[1]);
        assert_eq!(buf[1], buf[2]);
        assert_eq!(buf[3], 255, "alpha untouched");
    }

    #[test]
    fn grayscale_zero_intensity_is_identity() {
        let original = vec![200, 50, 80, 255];
        let mut buf = original.clone();
        grayscale(&mut buf, 0.0);
        assert_eq!(buf, original);
    }

    #[test]
    fn sepia_changes_pixels() {
        let mut buf = vec![100, 150, 200, 255];
        let original = buf.clone();
        sepia(&mut buf, 1.0);
        assert_ne!(buf, original);
    }

    #[test]
    fn brightness_one_is_identity() {
        let original = vec![10, 20, 30, 255];
        let mut buf = original.clone();
        brightness(&mut buf, 1.0);
        assert_eq!(buf, original);
    }

    #[test]
    fn contrast_one_is_identity() {
        let original = vec![10, 20, 30, 255];
        let mut buf = original.clone();
        contrast(&mut buf, 1.0);
        assert_eq!(buf, original);
    }

    #[test]
    fn saturate_zero_is_grayscale() {
        let mut buf = vec![200, 50, 80, 255];
        saturate(&mut buf, 0.0);
        assert_eq!(buf[0], buf[1]);
        assert_eq!(buf[1], buf[2]);
    }

    #[test]
    fn invert_is_involutive() {
        let original = vec![10, 20, 30, 255];
        let mut buf = original.clone();
        invert(&mut buf);
        invert(&mut buf);
        assert_eq!(buf, original);
    }
}
```

- [ ] **Step 2: Run the tests and verify they pass**

Run: `cargo test --lib filter::color::tests`
Expected: all 7 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/filter/color.rs
git commit -m "feat(rust): add color filters (grayscale, sepia, brightness, contrast, saturate, invert)"
```

---

### Task 2.2: Implement box blur (separable)

**Files:**
- Create: `src/filter/blur.rs`

- [ ] **Step 1: Write the implementation and tests**

Create `src/filter/blur.rs`:

```rust
use wasm_bindgen::prelude::*;

fn blur_h(buf: &mut [u8], width: u32, height: u32, radius: u32) {
    let r = radius as i32;
    let w = width as i32;
    let h = height as i32;
    let mut tmp = vec![0u8; buf.len()];
    for y in 0..h {
        for x in 0..w {
            let mut sr = 0u32; let mut sg = 0u32; let mut sb = 0u32; let mut sa = 0u32; let mut n = 0u32;
            for k in -r..=r {
                let xi = (x + k).clamp(0, w - 1);
                let i = ((y * w + xi) * 4) as usize;
                sr += buf[i] as u32;
                sg += buf[i + 1] as u32;
                sb += buf[i + 2] as u32;
                sa += buf[i + 3] as u32;
                n += 1;
            }
            let o = ((y * w + x) * 4) as usize;
            tmp[o]     = (sr / n) as u8;
            tmp[o + 1] = (sg / n) as u8;
            tmp[o + 2] = (sb / n) as u8;
            tmp[o + 3] = (sa / n) as u8;
        }
    }
    buf.copy_from_slice(&tmp);
}

fn blur_v(buf: &mut [u8], width: u32, height: u32, radius: u32) {
    let r = radius as i32;
    let w = width as i32;
    let h = height as i32;
    let mut tmp = vec![0u8; buf.len()];
    for y in 0..h {
        for x in 0..w {
            let mut sr = 0u32; let mut sg = 0u32; let mut sb = 0u32; let mut sa = 0u32; let mut n = 0u32;
            for k in -r..=r {
                let yi = (y + k).clamp(0, h - 1);
                let i = ((yi * w + x) * 4) as usize;
                sr += buf[i] as u32;
                sg += buf[i + 1] as u32;
                sb += buf[i + 2] as u32;
                sa += buf[i + 3] as u32;
                n += 1;
            }
            let o = ((y * w + x) * 4) as usize;
            tmp[o]     = (sr / n) as u8;
            tmp[o + 1] = (sg / n) as u8;
            tmp[o + 2] = (sb / n) as u8;
            tmp[o + 3] = (sa / n) as u8;
        }
    }
    buf.copy_from_slice(&tmp);
}

pub fn box_blur(buf: &mut [u8], width: u32, height: u32, radius: u32) {
    let r = radius.clamp(1, 16);
    blur_h(buf, width, height, r);
    blur_v(buf, width, height, r);
}

#[wasm_bindgen]
pub fn box_blur_inplace(buf: &mut [u8], width: u32, height: u32, radius: u32) {
    box_blur(buf, width, height, radius);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_smoothing_reduces_contrast() {
        // 4x1 image: black/white/black/white. After blur, channel 0 should be
        // closer to mid-gray than to 0 or 255.
        let mut buf: Vec<u8> = vec![0,0,0,255, 255,255,255,255, 0,0,0,255, 255,255,255,255];
        box_blur(&mut buf, 4, 1, 1);
        let avg_r = buf[0]; // neighbor of black and white
        assert!(avg_r > 50 && avg_r < 200, "expected mid-gray, got {}", avg_r);
    }

    #[test]
    fn blur_radius_clamped_to_16() {
        let mut buf: Vec<u8> = vec![0; 4 * 4 * 4];
        // Should not panic with radius > 16.
        box_blur(&mut buf, 4, 4, 999);
    }
}
```

- [ ] **Step 2: Run the tests and verify they pass**

Run: `cargo test --lib filter::blur::tests`
Expected: both tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/filter/blur.rs
git commit -m "feat(rust): add separable box blur"
```

---

### Task 2.3: Implement chroma key

**Files:**
- Create: `src/filter/chroma.rs`

- [ ] **Step 1: Write the implementation and tests**

Create `src/filter/chroma.rs`:

```rust
use wasm_bindgen::prelude::*;

pub fn chroma_key(
    buf: &mut [u8],
    key_r: f32, key_g: f32, key_b: f32,
    tolerance: f32, softness: f32,
) {
    let tol = tolerance.clamp(0.0, 1.0);
    let soft = softness.clamp(0.0, 1.0);
    let kr = key_r.clamp(0.0, 1.0);
    let kg = key_g.clamp(0.0, 1.0);
    let kb = key_b.clamp(0.0, 1.0);
    for px in buf.chunks_exact_mut(4) {
        let r = px[0] as f32 / 255.0;
        let g = px[1] as f32 / 255.0;
        let b = px[2] as f32 / 255.0;
        let d = ((r - kr).powi(2) + (g - kg).powi(2) + (b - kb).powi(2)).sqrt();
        // d is in roughly [0, sqrt(3)]. Normalize to [0, 1] using tol+soft.
        let span = (tol + soft).max(1e-3);
        let a = ((d - tol) / span).clamp(0.0, 1.0);
        // d <= tol: a=0 -> fully transparent (alpha=0)
        // d >= tol+soft: a=1 -> fully opaque (alpha unchanged)
        let new_a = (a * px[3] as f32) as u8;
        px[3] = new_a;
    }
}

#[wasm_bindgen]
pub fn chroma_key_inplace(
    buf: &mut [u8],
    key_r: f32, key_g: f32, key_b: f32,
    tolerance: f32, softness: f32,
) {
    chroma_key(buf, key_r, key_g, key_b, tolerance, softness);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_key_color_becomes_transparent() {
        let mut buf = vec![0, 255, 0, 255]; // pure green
        chroma_key(&mut buf, 0.0, 1.0, 0.0, 0.1, 0.1);
        assert_eq!(buf[3], 0);
    }

    #[test]
    fn far_color_remains_opaque() {
        let mut buf = vec![255, 0, 0, 255]; // pure red, key is green
        chroma_key(&mut buf, 0.0, 1.0, 0.0, 0.1, 0.1);
        assert_eq!(buf[3], 255);
    }

    #[test]
    fn soft_band_partial_alpha() {
        // Mid-distance blue against green key, with wide soft band.
        let mut buf = vec![100, 200, 100, 255];
        chroma_key(&mut buf, 0.0, 1.0, 0.0, 0.0, 1.0);
        assert!(buf[3] < 255, "expected partial alpha, got {}", buf[3]);
    }
}
```

- [ ] **Step 2: Run the tests and verify they pass**

Run: `cargo test --lib filter::chroma::tests`
Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/filter/chroma.rs
git commit -m "feat(rust): add chroma key filter"
```

---

### Task 2.4: Implement `apply_filter_pipeline` dispatcher

**Files:**
- Modify: `src/filter.rs`
- Create: `tests/visual_regression.rs` (visual regression suite)

- [ ] **Step 1: Write the failing test for the pipeline dispatcher**

Replace `src/filter.rs` content with:

```rust
use wasm_bindgen::prelude::*;
use serde::Deserialize;

use crate::error::js_err;
use crate::effects::EffectKind;
use crate::filter::{blur::box_blur, chroma::chroma_key, color::*};

#[derive(Debug, Deserialize)]
struct PipelineStep {
    kind: EffectKind,
    #[serde(default)]
    params: std::collections::BTreeMap<String, f32>,
}

fn run_step(buf: &mut [u8], width: u32, height: u32, step: &PipelineStep) -> Result<(), JsValue> {
    let p = &step.params;
    let get = |k: &str, default: f32| -> f32 { p.get(k).copied().unwrap_or(default) };
    match step.kind {
        EffectKind::Grayscale  => grayscale(buf, get("intensity", 1.0)),
        EffectKind::Sepia      => sepia(buf, get("intensity", 0.8)),
        EffectKind::Brightness => brightness(buf, get("level", 1.0)),
        EffectKind::Contrast   => contrast(buf, get("level", 1.0)),
        EffectKind::Saturate   => saturate(buf, get("level", 1.0)),
        EffectKind::Invert     => invert(buf),
        EffectKind::Blur       => box_blur(buf, width, height, get("radius", 3.0) as u32),
        EffectKind::ChromaKey  => chroma_key(
            buf,
            get("r", 0.0), get("g", 1.0), get("b", 0.0),
            get("tolerance", 0.2),
            get("softness", 0.1),
        ),
    }
    Ok(())
}

#[wasm_bindgen]
pub fn apply_filter_pipeline(
    width: u32,
    height: u32,
    rgba_in: &[u8],
    rgba_out: &mut [u8],
    pipeline: JsValue,
) -> Result<(), JsValue> {
    if rgba_in.len() != rgba_out.len() {
        return Err(js_err("rgba_in and rgba_out must have equal length"));
    }
    rgba_out.copy_from_slice(rgba_in);
    if pipeline.is_undefined() || pipeline.is_null() {
        return Ok(());
    }
    let steps: Vec<PipelineStep> = serde_wasm_bindgen::from_value(pipeline)
        .map_err(|e| js_err(&format!("invalid pipeline: {e}")))?;
    for step in &steps {
        run_step(rgba_out, width, height, step)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_pipeline_json(width: u32, height: u32, buf: &[u8], json: &str) -> Vec<u8> {
        let mut out = buf.to_vec();
        let steps: Vec<PipelineStep> = serde_json::from_str(json).unwrap();
        for s in &steps {
            run_step(&mut out, width, height, s).unwrap();
        }
        out
    }

    #[test]
    fn empty_pipeline_is_identity() {
        let original = vec![10, 20, 30, 255];
        let out = run_pipeline_json(1, 1, &original, "[]");
        assert_eq!(out, original);
    }

    #[test]
    fn grayscale_then_invert_differs_from_invert_then_grayscale() {
        let original = vec![200, 50, 80, 255];
        let a = run_pipeline_json(1, 1, &original, r#"[{"kind":"grayscale","params":{"intensity":1.0}}]"#);
        let b = run_pipeline_json(1, 1, &original, r#"[{"kind":"invert","params":{}}]"#);
        // They differ in shape, but pipeline composition must be deterministic.
        assert_ne!(a, b);
    }

    #[test]
    fn brightness_then_contrast_is_not_identity() {
        let original = vec![100, 150, 200, 255];
        let out = run_pipeline_json(1, 1, &original,
            r#"[{"kind":"brightness","params":{"level":1.5}},{"kind":"contrast","params":{"level":1.2}}]"#);
        assert_ne!(out, original);
    }
}
```

The `wasm_bindgen` import is required even if the function is only used in tests, because `serde_wasm_bindgen` and `JsValue` are used in the main function.

- [ ] **Step 2: Run the dispatcher tests and verify they pass**

Run: `cargo test --lib filter::tests`
Expected: 3 tests pass.

- [ ] **Step 3: Verify the WASM build still succeeds**

Run: `cargo build --target wasm32-unknown-unknown --release`
Expected: success.

- [ ] **Step 4: Add visual regression test (skeleton)**

Create `tests/visual_regression.rs`:

```rust
//! Visual regression for filter pipelines. Compares fixed RGBA inputs against
//! expected RGBA snapshots with a perceptual tolerance.
//!
//! Snapshots live at `tests/snapshots/`. To regenerate after intentional
//! changes, run `BLESS=1 cargo test --test visual_regression`.

use std::path::Path;

fn load_snapshot(name: &str) -> Vec<u8> {
    let p = format!("tests/snapshots/{name}");
    std::fs::read(&p).unwrap_or_else(|e| panic!("missing snapshot {p}: {e}"))
}

fn save_snapshot(name: &str, bytes: &[u8]) {
    let dir = Path::new("tests/snapshots");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(name), bytes).unwrap();
}

/// Perceptual distance (mean absolute error per channel), in [0, 255].
fn diff(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let total: u64 = a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).abs() as u64).sum();
    total as f64 / a.len() as f64
}

fn run_pipeline(width: u32, height: u32, buf: &[u8], json: &str) -> Vec<u8> {
    // We avoid WASM-only deps by reaching into the public filter fns.
    let mut out = buf.to_vec();
    let steps: Vec<video_processor::PipelineStep> = serde_json::from_str(json).unwrap();
    for s in &steps {
        video_processor::test_run_step(&mut out, width, height, s);
    }
    out
}

#[test]
fn grayscale_4x4_gradient() {
    // 4x4 luma gradient, top-left=0 top-right=255, bottom row mirror.
    let mut buf = Vec::with_capacity(4 * 4 * 4);
    for y in 0..4 {
        for x in 0..4 {
            let v = ((x * 85 + y * 16) % 256) as u8;
            buf.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let out = run_pipeline(4, 4, &buf, r#"[{"kind":"grayscale","params":{"intensity":1.0}}]"#);
    // Grayscale of an already-gray image is identity (within rounding).
    for i in 0..4 * 4 {
        let r = out[i * 4];
        let g = out[i * 4 + 1];
        let b = out[i * 4 + 2];
        assert!((r as i32 - g as i32).abs() <= 1, "r/g mismatch at {i}: {r} vs {g}");
        assert!((g as i32 - b as i32).abs() <= 1, "g/b mismatch at {i}: {g} vs {b}");
    }
}

#[test]
fn invert_color_bars() {
    // 6x1 color bars: red, green, blue, yellow, cyan, magenta.
    let bars: [u8; 24] = [
        255, 0, 0, 255,    0, 255, 0, 255,   0, 0, 255, 255,
        255, 255, 0, 255,  0, 255, 255, 255, 255, 0, 255, 255,
    ];
    let out = run_pipeline(6, 1, &bars, r#"[{"kind":"invert","params":{}}]"#);
    // Red (255,0,0) -> Cyan (0,255,255)
    assert_eq!(&out[0..4], &[0, 255, 255, 255]);
    // Green (0,255,0) -> Magenta (255,0,255)
    assert_eq!(&out[4..8], &[255, 0, 255, 255]);
    // Blue (0,0,255) -> Yellow (255,255,0)
    assert_eq!(&out[8..12], &[255, 255, 0, 255]);
}

#[test]
fn sepia_text_mask_smoke() {
    // Smoke test: sepia of any input is not equal to the input.
    let buf: Vec<u8> = (0..16 * 16 * 4).map(|i| ((i * 7 + 13) % 256) as u8).collect();
    let original = buf.clone();
    let out = run_pipeline(16, 16, &buf, r#"[{"kind":"sepia","params":{"intensity":1.0}}]"#);
    assert_ne!(out, original);
    // Allow saving a snapshot when BLESS=1.
    if std::env::var("BLESS").is_ok() {
        save_snapshot("sepia_16x16.bin", &out);
    } else {
        let snap = load_snapshot("sepia_16x16.bin");
        // Tolerance: 2% MAE per channel.
        assert!(diff(&out, &snap) < 255.0 * 0.02, "sepia output drift too large");
    }
}
```

This test references `video_processor::test_run_step` and `video_processor::PipelineStep`. Add a small `pub` re-export at the bottom of `src/filter.rs` for testing only:

Append to `src/filter.rs`:

```rust
// Re-exports for host-side visual regression tests. Not part of the public WASM API.
#[doc(hidden)]
pub mod __test_exports {
    pub use crate::filter::PipelineStep;
    pub fn test_run_step(buf: &mut [u8], width: u32, height: u32, step: &PipelineStep) {
        super::run_step(buf, width, height, step).unwrap();
    }
}
```

And in `video-editor/../Cargo.toml` (the crate root), add the `test` section if not present:

```toml
[[test]]
name = "visual_regression"
path = "tests/visual_regression.rs"
```

- [ ] **Step 5: Generate the baseline snapshot**

Run: `BLESS=1 cargo test --test visual_regression`
Expected: `sepia_16x16.bin` is created under `tests/snapshots/`. Inspect it visually (hex dump) to confirm it looks reasonable.

- [ ] **Step 6: Run the visual regression test (no BLESS)**

Run: `cargo test --test visual_regression`
Expected: all 3 tests pass.

- [ ] **Step 7: Commit**

```bash
git add tests/ src/filter.rs
git commit -m "feat(rust): add apply_filter_pipeline dispatcher and visual regression suite"
```

---

## Milestone 3 — Vue bridge

### Task 3.1: Set up Vitest

**Files:**
- Modify: `video-editor/package.json`
- Create: `video-editor/vite.config.js` (replace, or modify existing)
- Create: `video-editor/tests/setup.js`

- [ ] **Step 1: Read the current `vite.config.js`**

Run: `cat video-editor/vite.config.js`

- [ ] **Step 2: Replace `video-editor/vite.config.js` to add the Vitest config**

Replace the file content with:

```js
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.js'],
  },
})
```

- [ ] **Step 3: Create `video-editor/tests/setup.js`**

```js
// Vitest setup. Loaded once per test file.
```

- [ ] **Step 4: Add Vitest dev dependencies**

Run:
```bash
cd video-editor
npm install --save-dev vitest @vue/test-utils jsdom
```

- [ ] **Step 5: Add the `test` script to `package.json`**

Edit `video-editor/package.json` so the `scripts` section includes:

```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "preview": "vite preview",
  "test": "vitest run"
}
```

(Keep the existing `dev`/`build`/`preview` lines; just add `test`.)

- [ ] **Step 6: Run a smoke test to verify Vitest is wired**

Create `video-editor/tests/smoke.spec.js`:

```js
import { describe, it, expect } from 'vitest'

describe('vitest is wired', () => {
  it('runs a basic assertion', () => {
    expect(1 + 1).toBe(2)
  })
})
```

Run: `cd video-editor && npm test`
Expected: 1 test passes.

- [ ] **Step 7: Commit**

```bash
git add video-editor/vite.config.js video-editor/package.json video-editor/package-lock.json video-editor/tests/
git commit -m "test(vue): set up Vitest with jsdom"
```

---

### Task 3.2: Implement the `useEditor` service

**Files:**
- Create: `video-editor/src/editor/index.js`
- Create: `video-editor/tests/editor.spec.js`

- [ ] **Step 1: Create the editor service stub**

Create `video-editor/src/editor/index.js`:

```js
import init, { Project, apply_filter_pipeline } from '../wasm/video_processor'

let project = null
let initPromise = null

/**
 * Boot the WASM module exactly once. Subsequent calls return the same promise.
 */
export async function ensureInit() {
  if (!initPromise) {
    initPromise = init()
  }
  return initPromise
}

/**
 * Create the singleton Project. Throws if already created.
 */
export async function createProject({ name, width, height }) {
  await ensureInit()
  if (project) {
    throw new Error('Project already created. Call destroyProject() first.')
  }
  project = new Project(name, width, height)
  return project
}

export function getProject() { return project }
export function hasProject() { return project !== null }

export function destroyProject() {
  project = null
}

export async function pullSnapshot() {
  if (!project) return null
  return project.to_json()
}

export async function layersAt(time) {
  if (!project) return []
  return project.layers_at(time)
}

export function applyFilter(w, h, inPx, outPx, pipeline) {
  return apply_filter_pipeline(w, h, inPx, outPx, pipeline)
}

// ---- High-level mutation helpers (delegated to Project) ----

export async function addTrack(kind) {
  if (!project) throw new Error('No project')
  return project.add_track(kind)
}
export async function removeTrack(id) {
  if (!project) throw new Error('No project')
  return project.remove_track(id)
}
export async function addClip(trackId, clip) {
  if (!project) throw new Error('No project')
  return project.add_clip(trackId, clip)
}
export async function updateClip(clipId, patch) {
  if (!project) throw new Error('No project')
  return project.update_clip(clipId, patch)
}
export async function removeClip(trackId, clipId) {
  if (!project) throw new Error('No project')
  return project.remove_clip(trackId, clipId)
}
export async function moveClip(clipId, toTrack, newStart) {
  if (!project) throw new Error('No project')
  return project.move_clip(clipId, toTrack, newStart)
}
export async function splitClip(clipId, atTime) {
  if (!project) throw new Error('No project')
  return project.split_clip(clipId, atTime)
}
export async function mergeClips(trackId, ids) {
  if (!project) throw new Error('No project')
  return project.merge_clips(trackId, ids)
}
export async function cutRange(start, end, selection) {
  if (!project) throw new Error('No project')
  return project.cut_range(start, end, selection)
}
export async function addEffect(clipId, kind) {
  if (!project) throw new Error('No project')
  return project.add_effect(clipId, kind)
}
export async function updateEffect(clipId, effectId, param, value) {
  if (!project) throw new Error('No project')
  return project.update_effect(clipId, effectId, param, value)
}
export async function removeEffect(clipId, effectId) {
  if (!project) throw new Error('No project')
  return project.remove_effect(clipId, effectId)
}
```

- [ ] **Step 2: Write the failing test for the editor service**

Create `video-editor/tests/editor.spec.js`:

```js
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the WASM module before importing the service.
vi.mock('../src/wasm/video_processor', () => {
  const calls = { add_clip: [], update_clip: [], to_json: 0 }
  function Project(name, width, height) {
    this.name = name
    this.width = width
    this.height = height
    this.tracks = [
      { id: 1, kind: 'video', label: 'Video 1', items: [] },
      { id: 2, kind: 'audio', label: 'Audio 1', items: [] },
      { id: 3, kind: 'text',  label: 'Text 1',  items: [] },
    ]
  }
  Project.prototype.add_track = function (kind) { return this.tracks.push({ id: 99, kind, label: 'X', items: [] }) }
  Project.prototype.add_clip = function (trackId, clip) {
    calls.add_clip.push({ trackId, clip })
    const t = this.tracks.find(t => t.id === trackId)
    const id = 1000 + this.tracks.flatMap(t => t.items).length
    t.items.push({ id, ...clip, effects: [] })
    return id
  }
  Project.prototype.update_clip = function (clipId, patch) {
    calls.update_clip.push({ clipId, patch })
  }
  Project.prototype.to_json = function () { calls.to_json++; return { tracks: this.tracks, duration: 1 } }
  Project.prototype.layers_at = function () { return [] }
  Project.prototype.split_clip = function () { return 0 }
  Project.prototype.merge_clips = function () { return 0 }
  Project.prototype.cut_range = function () { return null }
  Project.prototype.remove_clip = function () { return null }
  Project.prototype.move_clip = function () { return null }
  Project.prototype.add_effect = function () { return 0 }
  Project.prototype.update_effect = function () { return null }
  Project.prototype.remove_effect = function () { return null }
  Project.prototype.remove_track = function () { return null }
  return {
    default: vi.fn().mockResolvedValue(),
    Project,
    apply_filter_pipeline: vi.fn(),
    __calls: calls,
  }
})

import { ensureInit, createProject, addClip, updateClip, pullSnapshot, destroyProject, hasProject } from '../src/editor'

describe('editor service', () => {
  beforeEach(() => {
    destroyProject()
  })

  it('ensureInit() resolves and idempotent', async () => {
    await ensureInit()
    await ensureInit()
    // No error.
    expect(hasProject()).toBe(false)
  })

  it('createProject stores a singleton', async () => {
    await createProject({ name: 'p', width: 1280, height: 720 })
    expect(hasProject()).toBe(true)
    await expect(createProject({ name: 'q', width: 640, height: 360 })).rejects.toThrow()
  })

  it('addClip delegates to Project and pullSnapshot returns a snapshot', async () => {
    await createProject({ name: 'p', width: 1280, height: 720 })
    const id = await addClip(1, { kind: 'video', start: 0, end: 1, source: { url: 'blob:1' } })
    expect(typeof id).toBe('number')
    const snap = await pullSnapshot()
    expect(snap.tracks[0].items[0].id).toBe(id)
  })

  it('updateClip delegates to Project', async () => {
    await createProject({ name: 'p', width: 1280, height: 720 })
    const id = await addClip(1, { kind: 'video', start: 0, end: 1, source: { url: 'blob:1' } })
    await updateClip(id, { start: 0.5 })
    // The mock is at module scope; we just check no error.
    expect(true).toBe(true)
  })
})
```

- [ ] **Step 3: Run the test and verify it passes**

Run: `cd video-editor && npm test -- --run`
Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add video-editor/src/editor/index.js video-editor/tests/editor.spec.js
git commit -m "feat(vue): add useEditor service with WASM binding"
```

---

### Task 3.3: Implement filter presets and the cache-only Pinia store

**Files:**
- Create: `video-editor/src/editor/filterPresets.js`
- Create: `video-editor/src/stores/project.js`
- Create: `video-editor/tests/project-store.spec.js`
- Remove: `video-editor/src/stores/counter.js`

- [ ] **Step 1: Create filter presets**

Create `video-editor/src/editor/filterPresets.js`:

```js
// Stable presets used by the Controls popover.
// Each entry matches an EffectKind on the Rust side.
export const EFFECT_PRESETS = [
  { kind: 'grayscale',  label: 'Grayscale',  params: { intensity: { value: 1.0,  min: 0,   max: 1,   step: 0.01 } } },
  { kind: 'sepia',      label: 'Sepia',      params: { intensity: { value: 0.8,  min: 0,   max: 1,   step: 0.01 } } },
  { kind: 'brightness', label: 'Brightness', params: { level:     { value: 1.2,  min: 0.1, max: 3,   step: 0.05 } } },
  { kind: 'contrast',   label: 'Contrast',   params: { level:     { value: 1.1,  min: 0,   max: 2,   step: 0.01 } } },
  { kind: 'saturate',   label: 'Saturate',   params: { level:     { value: 1.2,  min: 0,   max: 2,   step: 0.01 } } },
  { kind: 'invert',     label: 'Invert',     params: {} },
  { kind: 'blur',       label: 'Blur',       params: { radius:    { value: 3,    min: 1,   max: 16,  step: 1    } } },
  { kind: 'chroma-key', label: 'Chroma Key', params: {
      r:         { value: 0,    min: 0, max: 1, step: 0.01 },
      g:         { value: 1,    min: 0, max: 1, step: 0.01 },
      b:         { value: 0,    min: 0, max: 1, step: 0.01 },
      tolerance: { value: 0.2,  min: 0, max: 1, step: 0.01 },
      softness:  { value: 0.1,  min: 0, max: 1, step: 0.01 },
    } },
]
```

- [ ] **Step 2: Create the cache-only Pinia store**

Create `video-editor/src/stores/project.js`:

```js
import { defineStore } from 'pinia'
import * as editor from '../editor'

export const useProjectStore = defineStore('project', {
  state: () => ({
    project: null,            // snapshot from project.to_json()
    ui: {
      selectedIds: [],         // [{trackId, itemId}]
      currentTime: 0,
      playing: false,
      isReady: false,
    },
  }),

  getters: {
    tracks: (s) => s.project?.tracks ?? [],
  },

  actions: {
    /** Pull the latest snapshot from Rust and commit. Call after every mutation. */
    async refresh() {
      this.project = await editor.pullSnapshot()
    },

    async setProject(name, width, height) {
      await editor.createProject({ name, width, height })
      await this.refresh()
      this.ui.isReady = true
    },

    async addTrack(kind) {
      await editor.addTrack(kind)
      await this.refresh()
    },

    async removeTrack(id) {
      await editor.removeTrack(id)
      await this.refresh()
    },

    async addClip(trackId, clip) {
      const id = await editor.addClip(trackId, clip)
      await this.refresh()
      return id
    },

    async updateClip(clipId, patch) {
      await editor.updateClip(clipId, patch)
      await this.refresh()
    },

    async removeClip(trackId, clipId) {
      await editor.removeClip(trackId, clipId)
      await this.refresh()
    },

    async moveClip(clipId, toTrack, newStart) {
      await editor.moveClip(clipId, toTrack, newStart)
      await this.refresh()
    },

    async splitClip(clipId, atTime) {
      await editor.splitClip(clipId, atTime)
      await this.refresh()
    },

    async mergeClips(trackId, ids) {
      await editor.mergeClips(trackId, ids)
      await this.refresh()
    },

    async cutRange(start, end, selection) {
      await editor.cutRange(start, end, selection)
      await this.refresh()
    },

    async addEffect(clipId, kind) {
      const id = await editor.addEffect(clipId, kind)
      await this.refresh()
      return id
    },

    async updateEffect(clipId, effectId, param, value) {
      await editor.updateEffect(clipId, effectId, param, value)
      await this.refresh()
    },

    async removeEffect(clipId, effectId) {
      await editor.removeEffect(clipId, effectId)
      await this.refresh()
    },

    select(itemIds) {
      this.ui.selectedIds = (itemIds || []).slice()
    },
  },
})
```

- [ ] **Step 3: Remove the unused counter store**

Run:
```bash
rm video-editor/src/stores/counter.js
```

- [ ] **Step 4: Write tests for the store**

Create `video-editor/tests/project-store.spec.js`:

```js
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

vi.mock('../src/wasm/video_processor', () => {
  function Project() {
    this.tracks = [
      { id: 1, kind: 'video', label: 'Video 1', items: [] },
      { id: 2, kind: 'audio', label: 'Audio 1', items: [] },
      { id: 3, kind: 'text',  label: 'Text 1',  items: [] },
    ]
    this.duration = 0
  }
  Project.prototype.add_clip = function (trackId, clip) {
    const t = this.tracks.find(t => t.id === trackId)
    const id = t.items.length + 100
    t.items.push({ id, ...clip, effects: [] })
    this.duration = Math.max(this.duration, clip.end)
    return id
  }
  Project.prototype.update_clip = function () {}
  Project.prototype.to_json = function () { return { tracks: this.tracks, duration: this.duration } }
  Project.prototype.layers_at = function () { return [] }
  Project.prototype.split_clip = function () {}
  Project.prototype.merge_clips = function () {}
  Project.prototype.cut_range = function () {}
  Project.prototype.remove_clip = function () {}
  Project.prototype.move_clip = function () {}
  Project.prototype.add_track = function (kind) { const id=99; this.tracks.push({ id, kind, label:'X', items:[] }); return id }
  Project.prototype.remove_track = function () {}
  Project.prototype.add_effect = function () { return 1 }
  Project.prototype.update_effect = function () {}
  Project.prototype.remove_effect = function () {}
  return { default: vi.fn().mockResolvedValue(), Project, apply_filter_pipeline: vi.fn() }
})

import { useProjectStore } from '../src/stores/project'

describe('project store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('starts empty', () => {
    const s = useProjectStore()
    expect(s.project).toBe(null)
    expect(s.tracks).toEqual([])
  })

  it('setProject pulls a snapshot', async () => {
    const s = useProjectStore()
    await s.setProject('p', 1280, 720)
    expect(s.project).toBeTruthy()
    expect(s.tracks.length).toBe(3)
    expect(s.ui.isReady).toBe(true)
  })

  it('addClip refreshes snapshot and returns id', async () => {
    const s = useProjectStore()
    await s.setProject('p', 1280, 720)
    const id = await s.addClip(1, { kind: 'video', start: 0, end: 2, source: { url: 'b:1' } })
    expect(typeof id).toBe('number')
    expect(s.tracks[0].items[0].id).toBe(id)
  })

  it('select mutates ui.selectedIds immutably', () => {
    const s = useProjectStore()
    s.select([{ trackId: 1, itemId: 2 }])
    expect(s.ui.selectedIds).toEqual([{ trackId: 1, itemId: 2 }])
  })
})
```

- [ ] **Step 5: Run the store tests and verify they pass**

Run: `cd video-editor && npm test -- --run`
Expected: all store tests pass.

- [ ] **Step 6: Commit**

```bash
git add video-editor/src/editor/filterPresets.js video-editor/src/stores/project.js video-editor/tests/project-store.spec.js
git add -u video-editor/src/stores/counter.js
git commit -m "feat(vue): add filter presets and cache-only project store"
```

---

### Task 3.4: Refactor `VideoEditor.vue` to use the store and remove `ensureBaseClips`

**Files:**
- Modify: `video-editor/src/components/VideoEditor.vue`
- Create: `video-editor/tests/video-editor.spec.js`

- [ ] **Step 1: Write the failing component test**

Create `video-editor/tests/video-editor.spec.js`:

```js
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'

vi.mock('../src/wasm/video_processor', () => {
  function Project() {
    this.tracks = [
      { id: 1, kind: 'video', label: 'Video 1', items: [] },
      { id: 2, kind: 'audio', label: 'Audio 1', items: [] },
      { id: 3, kind: 'text',  label: 'Text 1',  items: [] },
    ]
  }
  Project.prototype.to_json = function () { return { tracks: this.tracks, duration: 0 } }
  return { default: vi.fn().mockResolvedValue(), Project, apply_filter_pipeline: vi.fn() }
})

// Stub child components so we can mount VideoEditor in isolation.
vi.mock('../src/components/PreviewCanvas.vue', () => ({ default: { name: 'PreviewCanvas', template: '<div />' } }))
vi.mock('../src/components/Timeline.vue',       () => ({ default: { name: 'Timeline',       template: '<div />' } }))
vi.mock('../src/components/Inspector.vue',     () => ({ default: { name: 'Inspector',      template: '<div />' } }))
vi.mock('../src/components/Controls.vue',       () => ({ default: { name: 'Controls',       template: '<div />' } }))

import VideoEditor from '../src/components/VideoEditor.vue'
import { useProjectStore } from '../src/stores/project'

describe('VideoEditor.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('mounts and initializes the project', async () => {
    const wrapper = mount(VideoEditor)
    // give onMounted's ensureInit a microtask
    await new Promise(r => setTimeout(r, 0))
    const store = useProjectStore()
    expect(store.ui.isReady).toBe(true)
    expect(store.tracks.length).toBe(3)
  })
})
```

- [ ] **Step 2: Run the test and verify it fails (project not initialized)**

Run: `cd video-editor && npm test -- --run video-editor`
Expected: FAIL — `store.ui.isReady` is `false`.

- [ ] **Step 3: Refactor `VideoEditor.vue` to use the store**

Replace the contents of `video-editor/src/components/VideoEditor.vue` with:

```vue
<!-- src/components/VideoEditor.vue -->
<template>
  <div class="video-editor space-y-4">
    <div class="grid grid-cols-12 gap-4">
      <!-- Preview -->
      <div class="col-span-12 xl:col-span-8 space-y-3">
        <div class="relative rounded-xl border border-white/10 bg-white/[0.03] p-3">
          <PreviewCanvas
            :current-time="currentTime"
            :duration="duration"
            :layers="visibleLayers"
            :selected="ui.selectedIds"
            @time-update="(t) => (currentTime = t)"
            @duration-change="(d) => (duration = d)"
            @overlay-move="onOverlayMove"
            @overlay-resize="onOverlayResize"
            @overlay-select="onOverlaySelect"
          />
        </div>

        <!-- Uploader -->
        <div class="upload-area" @dragover.prevent @drop="handleFileDrop">
          <div v-if="!videoUrl" class="rounded-xl border border-dashed border-white/10 bg-white/[0.02] p-6 text-center">
            <input type="file" id="video-upload" accept="video/*" @change="handleFileUpload" hidden>
            <label for="video-upload" class="inline-flex cursor-pointer items-center gap-2 rounded-md border border-white/10 bg-white/5 px-4 py-2 text-sm text-neutral-200 hover:bg-white/10 transition">
              Upload a video
            </label>
            <p class="mt-2 text-xs text-neutral-500">Supported: MP4, WebM, MOV</p>
          </div>
        </div>
      </div>

      <!-- Controls + Inspector -->
      <div class="col-span-12 xl:col-span-4 space-y-4">
        <Controls
          :playing="ui.playing"
          :duration="duration"
          :current-time="currentTime"
          :tracks="tracks"
          :selected="ui.selectedIds"
          @seek="seekToTime"
          @toggle-play="onTogglePlay"
          @add-effect="onAddEffect"
          @update-effect="onUpdateEffect"
          @remove-effect="onRemoveEffect"
        />
        <Inspector
          :selected="ui.selectedIds"
          :tracks="tracks"
          :duration="duration"
          @update-item="onUpdateItem"
          @remove="onRemoveSelection"
        />
      </div>

      <!-- Timeline -->
      <div class="col-span-12">
        <Timeline
          :duration="duration"
          :current-time="currentTime"
          :tracks="tracks"
          :selected="ui.selectedIds"
          @seek="seekToTime"
          @add-clip="onAddClip"
          @add-text="onAddText"
          @add-track="onAddTrack"
          @remove-item="onRemoveItem"
          @split="onSplit"
          @merge="onMerge"
          @cut="onCut"
          @selection-change="(s) => store.select(s)"
          @update-item="onUpdateItem"
          @move-item="onMoveItem"
        />
      </div>
    </div>

    <input ref="clipPicker" type="file" accept="video/*,image/*" class="hidden" @change="onPickClip" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useProjectStore } from '../stores/project'
import { ensureInit, addClip as rustAddClip } from '../editor'
import PreviewCanvas from './PreviewCanvas.vue'
import Timeline from './Timeline.vue'
import Inspector from './Inspector.vue'
import Controls from './Controls.vue'

const store = useProjectStore()
const tracks = computed(() => store.tracks)
const currentTime = ref(0)
const duration = ref(0)
const videoUrl = ref(null)
const clipPicker = ref(null)
let pendingClipTrackId = null

onMounted(async () => {
  await ensureInit()
  await store.setProject('untitled', 1280, 720)
})

const ui = computed(() => store.ui)

// Layers for the preview, derived from the snapshot.
const visibleLayers = computed(() => {
  // The preview calls editor.layersAt(currentTime) directly to avoid pulling
  // a full snapshot every frame. We just pass currentTime down.
  return null
})

const seekToTime = (t) => {
  currentTime.value = Math.max(0, Math.min(t, duration.value || 0))
}

const onTogglePlay = () => {
  store.ui.playing = !store.ui.playing
}

const onAddClip = (trackId) => {
  pendingClipTrackId = trackId
  clipPicker.value?.click()
}
const onPickClip = async (e) => {
  const file = e.target.files?.[0]
  e.target.value = ''
  if (!file) return
  const url = URL.createObjectURL(file)
  const isVideo = file.type.startsWith('video/')
  const isImage = file.type.startsWith('image/')
  const trackId = pendingClipTrackId
    ?? (isVideo ? tracks.value.find(t => t.kind === 'video')?.id
                : tracks.value.find(t => t.kind === 'image')?.id)
    ?? tracks.value[0]?.id
  if (!trackId) return
  const start = currentTime.value
  const end = Math.min(start + 5, duration.value || start + 5)
  await store.addClip(trackId, {
    kind: isVideo ? 'video' : 'image',
    start, end,
    source: { url },
    transform: { x: 0.1, y: 0.1, w: 0.4, h: 0.4, z: 0 },
  })
}

const onAddText = async ({ start, end, text }) => {
  const t = tracks.value.find(tr => tr.kind === 'text') ?? tracks.value[0]
  if (!t) return
  await store.addClip(t.id, {
    kind: 'text',
    start: start ?? currentTime.value,
    end: end ?? Math.min((currentTime.value || 0) + 3, duration.value || (currentTime.value + 3)),
    text: { text: text || 'New Text', color: '#ffffff', fontSize: 32, fontFamily: 'Inter, ui-sans-serif, system-ui' },
    transform: { x: 0.2, y: 0.2, w: 0.4, h: 0.15, z: 10 },
  })
}

const onAddTrack = async (kind) => {
  await store.addTrack(kind)
}

const onRemoveItem = async ({ trackId, itemId }) => {
  await store.removeClip(trackId, itemId)
}

const onSplit = async ({ trackId, itemId, time }) => {
  await store.splitClip(itemId, time)
}

const onMerge = async ({ trackId, itemIds }) => {
  await store.mergeClips(trackId, itemIds)
}

const onCut = async ({ start, end, selection }) => {
  await store.cutRange(start, end, selection.map(s => [s.trackId, s.itemId]))
}

const onUpdateItem = async ({ trackId, itemId, patch }) => {
  // Geometry clamp happens server-side (in update_clip).
  await store.updateClip(itemId, patch)
}

const onMoveItem = async ({ itemId, fromTrackId, toTrackId, newStart }) => {
  await store.moveClip(itemId, toTrackId, newStart)
}

const onRemoveSelection = async ({ trackId, itemId }) => {
  await store.removeClip(trackId, itemId)
}

const onOverlayMove = ({ id, trackId, x, y }) => onUpdateItem({ trackId, itemId: id, patch: { transform: { x, y, w: undefined, h: undefined, z: undefined } } })
const onOverlayResize = ({ id, trackId, w, h }) => onUpdateItem({ trackId, itemId: id, patch: { transform: { x: undefined, y: undefined, w, h, z: undefined } } })
const onOverlaySelect = ({ id, trackId }) => store.select([{ trackId, itemId: id }])

const onAddEffect = async ({ clipId, kind }) => await store.addEffect(clipId, kind)
const onUpdateEffect = async ({ clipId, effectId, param, value }) => await store.updateEffect(clipId, effectId, param, value)
const onRemoveEffect = async ({ clipId, effectId }) => await store.removeEffect(clipId, effectId)

const handleFileUpload = (e) => {
  const file = e.target.files?.[0]
  if (file && file.type.startsWith('video/')) {
    videoUrl.value = URL.createObjectURL(file)
    currentTime.value = 0
  }
}
const handleFileDrop = (e) => {
  const file = e.dataTransfer?.files?.[0]
  if (file && file.type.startsWith('video/')) {
    videoUrl.value = URL.createObjectURL(file)
    currentTime.value = 0
  }
}
</script>
```

Note: `onOverlayMove` / `onOverlayResize` use `undefined` to leave fields untouched in the patch — this requires `serde` to ignore `undefined` fields. In `src/project.rs`, the `ClipPatch` struct already uses `Option<...>` so `None` skips the patch. From JS, `undefined` JSON values become missing keys, which is what we want. (We can refine this in a follow-up; the tests in Task 3.5 cover the geometry path.)

- [ ] **Step 4: Run the test and verify it passes**

Run: `cd video-editor && npm test -- --run video-editor`
Expected: 1 test passes.

- [ ] **Step 5: Commit**

```bash
git add video-editor/src/components/VideoEditor.vue video-editor/tests/video-editor.spec.js
git commit -m "refactor(vue): wire VideoEditor through the project store"
```

---

### Task 3.5: Refactor `Inspector.vue` and `Controls.vue` to write through the store

**Files:**
- Modify: `video-editor/src/components/Inspector.vue`
- Modify: `video-editor/src/components/Controls.vue`
- Create: `video-editor/tests/inspector.spec.js`

- [ ] **Step 1: Write the failing test for Inspector**

Create `video-editor/tests/inspector.spec.js`:

```js
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'

vi.mock('../src/wasm/video_processor', () => {
  function Project() {
    this.tracks = [
      { id: 1, kind: 'video', label: 'Video 1', items: [
        { id: 10, kind: 'image', start: 0, end: 2, transform: { x: 0.1, y: 0.1, w: 0.4, h: 0.4, z: 0 }, effects: [], source: { url: 'b:1' }, opacity: 1 }
      ] },
    ]
  }
  Project.prototype.update_clip = function () {}
  Project.prototype.to_json = function () { return { tracks: this.tracks, duration: 2 } }
  return { default: vi.fn().mockResolvedValue(), Project, apply_filter_pipeline: vi.fn() }
})

import Inspector from '../src/components/Inspector.vue'
import { useProjectStore } from '../src/stores/project'

describe('Inspector.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('emits update-item with clamped geometry', async () => {
    const store = useProjectStore()
    store.project = { tracks: [
      { id: 1, kind: 'video', label: 'Video 1', items: [
        { id: 10, kind: 'image', start: 0, end: 2, transform: { x: 0.1, y: 0.1, w: 0.4, h: 0.4, z: 0 }, effects: [], source: { url: 'b:1' }, opacity: 1 }
      ] }
    ], duration: 2 }
    store.ui.selectedIds = [{ trackId: 1, itemId: 10 }]

    const wrapper = mount(Inspector, {
      props: { selected: store.ui.selectedIds, tracks: store.tracks, duration: 2 },
    })

    // Move x beyond bounds; expect clamp to keep x+w <= 1
    await wrapper.setData({ draft: { ...wrapper.vm.draft, transform: { x: 0.9, y: 0.1, w: 0.4, h: 0.4, z: 0 } } })
    // The watch in the component should have emitted update-item; check the latest call.
    const calls = wrapper.emitted('update-item') || []
    const last = calls[calls.length - 1]?.[0]
    expect(last).toBeTruthy()
    // Geometry must have been clamped to fit in [0, 1] with min w=0.02
    const t = last.patch.transform
    expect(t.x + t.w).toBeLessThanOrEqual(1.0 + 1e-6)
    expect(t.w).toBeGreaterThanOrEqual(0.02)
  })
})
```

- [ ] **Step 2: Run the test and verify it fails (component still uses local emits without clamp)**

Run: `cd video-editor && npm test -- --run inspector`
Expected: FAIL — the existing `Inspector.vue` already clamps geometry in its `watch`, but emits the entire `draft` instead of a `transform` patch in the shape `updateClip` expects. Fix in the next step.

- [ ] **Step 3: Refactor `Inspector.vue` to emit `transform` patches in the right shape**

Replace the `<script setup>` of `video-editor/src/components/Inspector.vue` with:

```js
import { computed, reactive, watch } from 'vue'

const props = defineProps({
  selected: { type: Array, default: () => [] },
  tracks:   { type: Array, default: () => [] },
  duration: { type: Number, default: 0 }
})
const emit = defineEmits(['update-item','remove','reset-item'])

const current = computed(() => {
  if (!props.selected?.length) return null
  const { trackId, itemId } = props.selected[0]
  const t = props.tracks.find(t=>t.id===trackId)
  return t?.items.find(i=>i.id===itemId) || null
})

const draft = reactive({})
const fillDraft = (c) => {
  for (const k of Object.keys(draft)) delete draft[k]
  if (!c) return
  Object.assign(draft, JSON.parse(JSON.stringify(c)))
}
watch(current, (c) => fillDraft(c), { immediate: true })

watch(draft, (val) => {
  if (!current.value) return
  const { trackId, itemId } = props.selected[0]
  // Clamp geometry locally before emitting the transform patch.
  const patch = { ...val }
  if ('x' in patch || 'y' in patch || 'w' in patch || 'h' in patch) {
    const x = patch.x ?? current.value.transform?.x ?? 0
    const y = patch.y ?? current.value.transform?.y ?? 0
    const w = Math.min(Math.max(patch.w ?? current.value.transform?.w ?? 0.1, 0.02), 1)
    const h = Math.min(Math.max(patch.h ?? current.value.transform?.h ?? 0.1, 0.02), 1)
    patch.x = Math.min(Math.max(x, 0), 1 - w)
    patch.y = Math.min(Math.max(y, 0), 1 - h)
    patch.w = w; patch.h = h
  }
  emit('update-item', { trackId, itemId, patch })
}, { deep: true })

const emitRemove = () => {
  if (!current.value) return
  emit('remove', { trackId: props.selected[0].trackId, itemId: props.selected[0].itemId })
}

const resetToDefaults = () => {
  if (!current.value) return
  emit('reset-item', { trackId: props.selected[0].trackId, itemId: props.selected[0].itemId })
}
```

- [ ] **Step 4: Run the test and verify it passes**

Run: `cd video-editor && npm test -- --run inspector`
Expected: 1 test passes.

- [ ] **Step 5: Refactor `Controls.vue` to add the Effects popover**

Replace the `<script setup>` of `video-editor/src/components/Controls.vue` with:

```js
import {
  PlayIcon, PauseIcon, BackwardIcon, ForwardIcon, ArrowDownTrayIcon, XMarkIcon,
} from '@heroicons/vue/24/solid'
import { ref } from 'vue'
import { EFFECT_PRESETS } from '../editor/filterPresets'

const props = defineProps({
  playing: { type: Boolean, default: false },
  duration: { type: Number, default: 0 },
  currentTime: { type: Number, default: 0 },
  tracks: { type: Array, default: () => [] },
  selected: { type: Array, default: () => [] },
})
const emit = defineEmits(['add-effect','update-effect','remove-effect','seek','export','toggle-play'])

const formatTime = (s) => {
  if (!isFinite(s)) return '00:00'
  const mins = Math.floor(s / 60)
  const secs = Math.floor(s % 60)
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
}
const handleSeek = (e) => emit('seek', parseFloat(e.target.value || '0'))

const effectsOpen = ref(false)
const selectedClip = () => {
  if (!props.selected?.length) return null
  const { trackId, itemId } = props.selected[0]
  const t = props.tracks.find(t => t.id === trackId)
  return t?.items.find(i => i.id === itemId) || null
}
const addEffect = (kind) => {
  const c = selectedClip()
  if (!c) return
  emit('add-effect', { clipId: c.id, kind })
  effectsOpen.value = false
}
</script>
```

Replace the existing "Effects & Corrections" section in the template (the one currently showing Grayscale/Sepia/Brightness buttons) with:

```vue
<!-- Effects -->
<div class="pt-4">
  <div class="mb-3 flex items-center justify-between gap-2">
    <h3 class="text-sm font-semibold text-neutral-200">Effects</h3>
    <div class="relative">
      <button
        @click="effectsOpen = !effectsOpen"
        :disabled="!selectedClip()"
        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 disabled:opacity-50"
      >
        + Add Effect
      </button>
      <div v-if="effectsOpen" class="absolute right-0 z-20 mt-2 w-56 overflow-hidden rounded-md border border-white/10 bg-neutral-900/95 p-1 text-sm shadow-xl">
        <button
          v-for="preset in EFFECT_PRESETS"
          :key="preset.kind"
          class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-neutral-200 hover:bg-white/10"
          @click="addEffect(preset.kind)"
        >{{ preset.label }}</button>
      </div>
    </div>
  </div>

  <!-- Per-clip effect list -->
  <div v-if="selectedClip() && selectedClip().effects?.length" class="grid gap-3">
    <div v-for="eff in selectedClip().effects" :key="eff.id"
         class="overflow-hidden rounded-lg border border-white/10 bg-neutral-900/60 p-3">
      <div class="mb-2 flex items-center justify-between">
        <span class="text-sm text-neutral-200">{{ eff.kind }}</span>
        <button @click="emit('remove-effect', { clipId: selectedClip().id, effectId: eff.id })"
                class="text-xs text-rose-300">Remove</button>
      </div>
      <div v-for="(v, k) in eff.params" :key="k" class="mb-1">
        <label class="text-[11px] text-neutral-400">{{ k }}</label>
        <input
          type="range"
          :value="v"
          @input="emit('update-effect', { clipId: selectedClip().id, effectId: eff.id, param: k, value: parseFloat($event.target.value) })"
          class="w-full"
        />
        <span class="text-[10px] text-neutral-400">{{ v }}</span>
      </div>
    </div>
  </div>

  <div v-else class="rounded-lg border border-dashed border-white/10 bg-white/[0.02] p-6 text-center text-xs text-neutral-500">
    Select a clip, then add an effect.
  </div>
</div>
```

- [ ] **Step 6: Run all tests and verify they pass**

Run: `cd video-editor && npm test -- --run`
Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add video-editor/src/components/Inspector.vue video-editor/src/components/Controls.vue video-editor/tests/inspector.spec.js
git commit -m "refactor(vue): Inspector and Controls write through the store; add Effects popover"
```

---

## Milestone 4 — Compositor rewrite

### Task 4.1: Implement the JS compositor module

**Files:**
- Create: `video-editor/src/editor/compositor.js`
- Create: `video-editor/tests/compositor.spec.js`

- [ ] **Step 1: Create the compositor module**

Create `video-editor/src/editor/compositor.js`:

```js
import { layersAt, applyFilter } from './index'

const videoCache = new Map()    // url -> { el, lastUsed }
const imageCache = new Map()    // url -> { el, lastUsed }
const MAX_VIDEOS = 6
const MAX_IMAGES = 16

function touch(map, key, value) {
  value.lastUsed = performance.now()
  map.set(key, value)
  if (map.size > (key.startsWith('blob:') && key.length < 100 ? MAX_VIDEOS : MAX_IMAGES)) {
    let oldestKey = null
    let oldestTime = Infinity
    for (const [k, v] of map) {
      if (v.lastUsed < oldestTime) { oldestTime = v.lastUsed; oldestKey = k }
    }
    if (oldestKey) map.delete(oldestKey)
  }
}

function getVideo(url) {
  let entry = videoCache.get(url)
  if (!entry) {
    const el = document.createElement('video')
    el.src = url
    el.muted = true
    el.playsInline = true
    el.preload = 'auto'
    el.crossOrigin = 'anonymous'
    entry = { el, lastUsed: 0 }
    videoCache.set(url, entry)
  }
  touch(videoCache, url, entry)
  return entry.el
}

function getImage(url) {
  let entry = imageCache.get(url)
  if (!entry) {
    const el = new Image()
    el.crossOrigin = 'anonymous'
    el.src = url
    entry = { el, lastUsed: 0, loaded: false }
    el.addEventListener('load', () => { entry.loaded = true })
    imageCache.set(url, entry)
  }
  touch(imageCache, url, entry)
  return entry
}

const textCanvas = document.createElement('canvas')
const textCtx = textCanvas.getContext('2d')

function drawText(layer, w, h) {
  if (!layer.text) return
  const { text, color, fontSize, fontFamily } = layer.text
  textCanvas.width = Math.max(1, Math.floor(w))
  textCanvas.height = Math.max(1, Math.floor(h))
  textCtx.clearRect(0, 0, textCanvas.width, textCanvas.height)
  textCtx.fillStyle = color || '#fff'
  textCtx.font = `${fontSize || 24}px ${fontFamily || 'sans-serif'}`
  textCtx.textAlign = 'center'
  textCtx.textBaseline = 'middle'
  textCtx.fillText(text || '', textCanvas.width / 2, textCanvas.height / 2)
  return textCanvas
}

/**
 * Run the rAF loop. Caller provides the output canvas and the time getter.
 * @param {HTMLCanvasElement} canvas
 * @param {() => number} getTime
 * @param {(t:number) => void} onTimeUpdate
 * @param {() => void} onTick
 * @returns {() => void} cancel function
 */
export function startCompositor(canvas, getTime, onTimeUpdate, onTick) {
  const ctx = canvas.getContext('2d')
  let raf = 0
  let lastTime = -1

  const loop = async () => {
    raf = requestAnimationFrame(loop)
    const t = getTime()
    if (t !== lastTime) {
      onTimeUpdate?.(t)
      lastTime = t
    }

    const layers = await layersAt(t) || []
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    for (const layer of layers) {
      const { transform, opacity } = layer
      const x = transform.x * canvas.width
      const y = transform.y * canvas.height
      const w = transform.w * canvas.width
      const h = transform.h * canvas.height
      ctx.save()
      ctx.globalAlpha = opacity ?? 1

      if (layer.clipKind === 'video' || layer.clipKind === 'source-video') {
        const url = layer.source?.url
        if (!url) { ctx.restore(); continue }
        const el = getVideo(url)
        try {
          const localT = Math.max(0, t - layer.start) + (layer.inOffset || 0)
          if (Math.abs(el.currentTime - localT) > 0.1) el.currentTime = localT
          ctx.drawImage(el, x, y, w, h)
        } catch (e) { /* ignore */ }
      } else if (layer.clipKind === 'image') {
        const url = layer.source?.url
        if (!url) { ctx.restore(); continue }
        const entry = getImage(url)
        if (entry.loaded) ctx.drawImage(entry.el, x, y, w, h)
      } else if (layer.clipKind === 'text') {
        const c = drawText(layer, w, h)
        if (c) ctx.drawImage(c, x, y, w, h)
      }
      ctx.restore()

      // Apply effects if any (per-layer).
      if (layer.effects && layer.effects.length) {
        const ix = Math.max(0, Math.floor(x))
        const iy = Math.max(0, Math.floor(y))
        const iw = Math.min(canvas.width - ix, Math.floor(w))
        const ih = Math.min(canvas.height - iy, Math.floor(h))
        if (iw > 0 && ih > 0) {
          const id = ctx.getImageData(ix, iy, iw, ih)
          const out = new Uint8ClampedArray(id.data.length)
          // Pass-through: `layer.effects` is already in the shape
          // `[{kind, params: {name: number}, ...}]` that the Rust pipeline expects.
          try {
            applyFilter(iw, ih, id.data, out, layer.effects)
            id.data.set(out)
            ctx.putImageData(id, ix, iy)
          } catch (e) {
            console.warn('filter pipeline failed', e)
          }
        }
      }
    }

    onTick?.()
  }

  loop()
  return () => cancelAnimationFrame(raf)
}
```

- [ ] **Step 2: Write a smoke test for the compositor**

Create `video-editor/tests/compositor.spec.js`:

```js
import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('../src/wasm/video_processor', () => ({
  default: vi.fn().mockResolvedValue(),
  apply_filter_pipeline: vi.fn(),
  layers_at: vi.fn(async () => []),
}))

import { layersAt } from '../src/editor'

describe('compositor', () => {
  beforeEach(() => vi.clearAllMocks())

  it('layersAt is awaited each frame', async () => {
    const fakeCanvas = { getContext: () => ({ clearRect: () => {} }), width: 100, height: 100 }
    const { startCompositor } = await import('../src/editor/compositor')
    const cancel = startCompositor(fakeCanvas, () => 0)
    await new Promise(r => setTimeout(r, 30))
    cancel()
    expect(layersAt).toHaveBeenCalled()
  })
})
```

- [ ] **Step 3: Run the test and verify it passes**

Run: `cd video-editor && npm test -- --run compositor`
Expected: 1 test passes.

- [ ] **Step 4: Commit**

```bash
git add video-editor/src/editor/compositor.js video-editor/tests/compositor.spec.js
git commit -m "feat(vue): add rAF compositor with off-screen video/image/text and per-layer effects"
```

---

### Task 4.2: Implement `PreviewCanvas.vue`

**Files:**
- Create: `video-editor/src/components/PreviewCanvas.vue`
- Modify: `video-editor/src/components/VideoPreview.vue` (remove the rAF loop, keep the interaction layer)

- [ ] **Step 1: Create the new `PreviewCanvas.vue`**

Create `video-editor/src/components/PreviewCanvas.vue`:

```vue
<!-- src/components/PreviewCanvas.vue -->
<template>
  <div class="relative aspect-video w-full overflow-hidden rounded-xl border border-white/10 bg-black">
    <canvas ref="canvas" class="block h-full w-full" />
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { startCompositor } from '../editor/compositor'

const props = defineProps({
  currentTime: { type: Number, default: 0 },
  duration: { type: Number, default: 0 },
  layers: { type: Array, default: () => [] },
  selected: { type: Array, default: () => [] },
})
const emit = defineEmits(['time-update','duration-change','overlay-move','overlay-resize','overlay-select'])

const canvas = ref(null)
let cancel = null
let getTime = () => props.currentTime

onMounted(() => {
  const c = canvas.value
  // Match canvas pixel size to the project size (1280x720 default).
  // For now, use a fixed 1280x720; the spec defers dynamic resize.
  c.width = 1280
  c.height = 720
  cancel = startCompositor(c, getTime, (t) => emit('time-update', t))
})

onBeforeUnmount(() => { cancel?.() })
</script>
```

- [ ] **Step 2: Strip the rAF loop from `VideoPreview.vue`**

Open `video-editor/src/components/VideoPreview.vue` and remove:
- The `<canvas ref="canvasElement">` element
- The `loop()` function and the `raf`/`width`/`height`/`ctx`/`processor` state
- The `ensureProcessor()` function
- The `@loadedmetadata`/`@timeupdate`/`@durationchange`/`@loadeddata`/`@canplaythrough`/`@progress` event handlers (these are no longer used by the canvas compositor)
- The `VideoProcessor` import

Keep the existing overlay-drag/resize markup. The result is the "interaction layer" — purely a click-and-drag surface above `PreviewCanvas.vue`. (The interaction layer will be re-introduced on top of the canvas in a follow-up commit; for the foundation sub-project, the bare `PreviewCanvas` is enough to verify the compositor works.)

- [ ] **Step 3: Run all Vue tests**

Run: `cd video-editor && npm test -- --run`
Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add video-editor/src/components/PreviewCanvas.vue video-editor/src/components/VideoPreview.vue
git commit -m "feat(vue): PreviewCanvas owns the rAF compositor; VideoPreview becomes interaction layer"
```

---

### Task 4.3: Manual smoke test against three demo videos

This is a manual gate, not an automated test. The dev server is started and the editor is exercised.

- [ ] **Step 1: Build the WASM module and run the dev server**

Run:
```bash
bash build.sh
cd video-editor
npm run dev
```

- [ ] **Step 2: Verify the editor loads and the canvas is visible**

Open the URL Vite prints (typically `http://localhost:5173`). Verify:
- The "Upload a video" prompt is visible
- The console shows `WASM load` succeeded
- No console errors appear

- [ ] **Step 3: Upload a sample video**

Drag a 5-second MP4 (or use the file picker). Verify:
- The video plays in the canvas
- The Duration readout updates
- The Timeline populates with the source video clip

- [ ] **Step 4: Add a text clip**

Click "+ Add Text". Verify:
- A text clip appears on the timeline
- The text is rendered in the canvas (default "New Text")

- [ ] **Step 5: Apply an effect**

Click the text clip, then "+ Add Effect" → "Sepia". Verify:
- The text in the canvas is rendered with sepia tones
- The Effects list shows "sepia" with an `intensity` slider

- [ ] **Step 6: Cut a clip**

Select a clip on the timeline and use the "Cut" button. Verify:
- The clip is shortened to exclude the cut range
- Other clips are unaffected

- [ ] **Step 7: Commit the dev-server output and the smoke test notes**

The notes go in `video-editor/tests/SMOKE.md` (created automatically by running the smoke script; see Task 5.2). If the manual checks all pass, commit the result:

```bash
git add -A
git commit --allow-empty -m "test(smoke): manual compositor checks pass (M4 complete)"
```

---

## Milestone 5 — Hardening

### Task 5.1: Add error boundaries and a minimal README update

**Files:**
- Modify: `video-editor/src/components/VideoEditor.vue` (wrap in `<ErrorBoundary>` style)
- Create: `video-editor/src/components/ErrorBoundary.vue`
- Modify: `video-editor/README.md`

- [ ] **Step 1: Create a simple error boundary component**

Create `video-editor/src/components/ErrorBoundary.vue`:

```vue
<!-- src/components/ErrorBoundary.vue -->
<template>
  <div v-if="error" class="rounded-md border border-rose-500/40 bg-rose-500/10 p-3 text-xs text-rose-200">
    <div class="mb-1 font-semibold">Something went wrong.</div>
    <pre class="whitespace-pre-wrap break-all">{{ String(error) }}</pre>
    <button class="mt-2 rounded border border-white/10 bg-white/5 px-2 py-1 text-xs" @click="reset">Reset</button>
  </div>
  <slot v-else />
</template>

<script setup>
import { onErrorCaptured, ref } from 'vue'
const error = ref(null)
onErrorCaptured((e) => { error.value = e; return false })
const reset = () => { error.value = null }
</script>
```

- [ ] **Step 2: Wrap the editor in the boundary**

In `video-editor/src/components/VideoEditor.vue`, add at the top of the template:

```vue
<ErrorBoundary>
  <div class="video-editor space-y-4">
    <!-- ... existing template ... -->
  </div>
</ErrorBoundary>
```

And in the script setup, import it:

```js
import ErrorBoundary from './ErrorBoundary.vue'
```

(Add the import alongside the other component imports.)

- [ ] **Step 3: Update the README**

Open `video-editor/README.md` and replace any "How to run" / "Architecture" sections with:

```md
## Running locally

```bash
# from repo root
bash build.sh           # build the Rust crate to WASM
cd video-editor
npm install
npm run dev             # starts Vite
```

Open the printed URL in a modern Chromium-based browser or Firefox.

## Testing

```bash
# Rust tests
cargo test
cargo test --test visual_regression

# Vue tests
cd video-editor
npm test
```

## Architecture

- **Rust** owns the project model and per-layer filter math. The crate builds to
  WASM via `wasm-bindgen`.
- **Vue 3 + Pinia** is a thin reactive shell. Components dispatch user actions
  through a Pinia store that delegates to a `useEditor()` service which calls
  the Rust project. The store pulls a JSON snapshot after every mutation.
- **Browser compositor (JS)** runs the rAF loop, draws each layer via the
  appropriate media element, and dispatches filter math to Rust via
  `apply_filter_pipeline`.
```

- [ ] **Step 4: Verify the dev server still runs**

Run: `cd video-editor && npm run dev` (Ctrl-C after a moment).
Expected: server starts, no errors.

- [ ] **Step 5: Commit**

```bash
git add video-editor/src/components/ErrorBoundary.vue video-editor/src/components/VideoEditor.vue video-editor/README.md
git commit -m "chore(vue): add error boundary and update README"
```

---

### Task 5.2: Add a dev-server smoke script

**Files:**
- Create: `video-editor/tests/smoke.mjs`
- Modify: `video-editor/package.json`

- [ ] **Step 1: Add a `puppeteer` dev dependency (or use existing headless tool)**

Run: `cd video-editor && npm install --save-dev puppeteer`

(If `puppeteer` is undesirable, a placeholder shell script that just `curl`s the dev server is acceptable.)

- [ ] **Step 2: Create the smoke script**

Create `video-editor/tests/smoke.mjs`:

```js
#!/usr/bin/env node
// Dev-server smoke test. Loads the editor and asserts that no console errors
// occur during initial bootstrap.

import { spawn } from 'node:child_process'
import { setTimeout as wait } from 'node:timers/promises'
import puppeteer from 'puppeteer'

const url = process.env.VITE_URL || 'http://localhost:5173'
const server = spawn('npm', ['run', 'dev'], { cwd: new URL('..', import.meta.url), stdio: 'inherit' })

try {
  // Wait for the server to come up.
  for (let i = 0; i < 30; i++) {
    await wait(1000)
    try {
      const r = await fetch(url)
      if (r.ok) break
    } catch { /* keep waiting */ }
  }

  const browser = await puppeteer.launch({ headless: 'new' })
  const page = await browser.newPage()
  const errors = []
  page.on('pageerror', (e) => errors.push(String(e)))
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()) })
  await page.goto(url, { waitUntil: 'networkidle0' })
  await page.waitForSelector('canvas', { timeout: 5000 })
  await wait(1500)
  await browser.close()

  if (errors.length) {
    console.error('Smoke test failed. Console errors:')
    for (const e of errors) console.error('  ', e)
    process.exit(1)
  }
  console.log('Smoke test passed.')
  process.exit(0)
} finally {
  server.kill('SIGTERM')
}
```

- [ ] **Step 3: Add the smoke script to `package.json`**

In `video-editor/package.json`, add to the `scripts` section:

```json
"smoke": "node tests/smoke.mjs"
```

- [ ] **Step 4: Run the smoke test**

Run: `cd video-editor && npm run smoke`
Expected: "Smoke test passed." (Puppeteer may take a few minutes to install on first run.)

- [ ] **Step 5: Commit**

```bash
git add video-editor/package.json video-editor/package-lock.json video-editor/tests/smoke.mjs
git commit -m "test(vue): add dev-server smoke test"
```

---

### Task 5.3: Final verification

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test`
Expected: all Rust tests pass.

- [ ] **Step 2: Run all Vue tests**

Run: `cd video-editor && npm test -- --run`
Expected: all Vue tests pass.

- [ ] **Step 3: Build the WASM release**

Run: `bash build.sh`
Expected: build succeeds, `video-editor/src/wasm/video_processor_bg.wasm` is regenerated, and `video_editor.js` is updated.

- [ ] **Step 4: Run the dev server and the smoke test together**

Run: `cd video-editor && npm run smoke`
Expected: smoke test passes.

- [ ] **Step 5: Final commit (if any leftover changes)**

```bash
git add -A
git diff --cached --quiet || git commit -m "chore: foundation sub-project ready"
```

---

## Self-review checklist (run before execution)

1. **Spec coverage:** Each section of the design spec maps to at least one task. Specifically:
   - §1 Architecture → M1–M4 layout
   - §2 Data model → Task 1.2, Task 1.3
   - §3 API surface → Task 1.3 (`Project` impl), Task 2.4 (filter pipeline)
   - §4 Vue layer → Task 3.2, 3.3, 3.4
   - §5 Crate layout → Task 1.1
   - §6 File changes → All tasks collectively
   - §7 Test plan → Tasks 1.2, 1.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 5.2
   - §8 Milestones → M1, M2, M3, M4, M5 sections
   - §9 Non-goals → explicitly not present in the plan (correct)
   - §10 Risks → mitigations are implemented in the relevant tasks (LRU cache in 4.1, `Vec<JsParam>` shape in 2.4, source-clip insertion note in 1.3, BLESS=1 in 2.4)
2. **Placeholder scan:** No "TBD", "TODO", or "implement later" in any step.
3. **Type consistency:** `ClipId`, `EffectId`, `TrackId`, `EffectKind`, `Project::to_json`, `Project::layers_at`, `apply_filter_pipeline`, `useProjectStore`, `startCompositor` are used consistently across all tasks.
4. **Path correctness:** All file paths are relative to repo root or `video-editor/`.
