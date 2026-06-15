# Foundation Sub-Project: Rust Project Model + Real-Time Compositor

**Date:** 2026-06-15
**Phase:** 1 of 4 (foundation)
**Goal:** Establish the Rust-owned project model, the data-oriented WASM API, and a real-time preview compositor that can drive every later phase (effects, transitions, keyframes, animation, audio, export).

## Why this sub-project exists

A "CapCut-like" web editor is the union of roughly ten subsystems. Trying to design all of them in one spec produces a document that is either too shallow to plan against or too large to review. The recommended path is to design and ship the *foundation* first — project model + compositor + filter pipeline — and treat the rest as separate spec → plan → implement cycles that build on top of it.

This phase produces a fully working editor in the sense that a user can:
- Import a video
- Add clips (video, image, text) to typed tracks
- Move, resize, split, merge, and cut clips
- Apply filters to individual clips
- See a correctly composited real-time preview
- See a JSON snapshot of the project that the export pipeline (Phase 3) will consume

It explicitly does **not** ship: real video export, audio mixing, transitions, animation/keyframes, undo/redo, project save/load. Those are tracked in §9 (Non-goals) and §11 (follow-up specs).

## 1. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Vue 3 UI (Pinia store is a thin cache)                      │
│  ──────────────────────────────────────────                  │
│  • Timeline / Inspector / Controls / PreviewCanvas           │
│  • Components dispatch user actions through a single         │
│    `useEditor()` composable.                                 │
│  • Pulls `project.to_json()` for snapshot rendering.         │
└────────────────────────┬─────────────────────────────────────┘
                         │  wasm-bindgen (data-oriented)
┌────────────────────────┴─────────────────────────────────────┐
│  Rust core (cdylib)                                          │
│  ─────────────────                                           │
│  project   – state + mutation API                            │
│  filter    – pure RGBA filter math (CPU)                     │
│  text      – placeholder for future rasterization hook        │
│  (no compositor here)                                        │
└────────────────────────┬─────────────────────────────────────┘
                         │
┌────────────────────────┴─────────────────────────────────────┐
│  Browser compositor (JS, rAF loop)                           │
│  ─────────────────────────────────                           │
│  • One <video> per source-video clip                         │
│  • One output <canvas>                                       │
│  • Per frame: ask Rust for visible layer descriptors,        │
│    rasterize text on offscreen canvas, draw video frames,    │
│    apply Rust filters, blend with alpha onto output          │
└──────────────────────────────────────────────────────────────┘
```

**Boundaries:**

- **Rust has no DOM access** for state queries. `web-sys` is used only for low-level types where strictly needed by the filter pipeline (this phase: none).
- **Vue has no direct WASM calls outside `useEditor()`.**
- **Compositor (JS) does not read or write Rust state** — only asks `project.layers_at(time) -> LayerDescriptor[]`.

**Why this matters for the future:** the same `layers_at()` query is what the export pipeline will call frame-by-frame in Phase 3, so adding WebCodecs + muxer later does not require rewriting the foundation.

## 2. Rust data model

```rust
// pseudocode
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub resolution: (u32, u32),       // width, height
    pub duration: f32,                // seconds; recomputed on edit
    pub tracks: Vec<Track>,
}

pub enum TrackKind { Video, Audio, Text, Image }

pub struct Track {
    pub id: TrackId,
    pub kind: TrackKind,
    pub label: String,
    pub items: Vec<Clip>,
}

pub enum ClipKind { SourceVideo, SourceAudio, Video, Image, Text }

pub struct Clip {
    pub id: ClipId,
    pub kind: ClipKind,
    pub start: f32,                   // timeline position (s)
    pub end: f32,                     // exclusive
    pub in_offset: f32,               // for media clips: offset into source
    pub transform: Transform,          // x,y,w,h normalized to 0..1; z is integer stacking index
    pub effects: Vec<EffectInstance>,
    pub source: Option<MediaSource>,  // src url or asset id
    pub text: Option<TextProps>,
    pub opacity: f32,                 // 0..1, defaults to 1
}

pub struct Transform { pub x: f32, pub y: f32, pub w: f32, pub h: f32, pub z: i32 }

pub struct EffectInstance {
    pub id: EffectId,
    pub kind: EffectKind,
    pub params: BTreeMap<String, f32>,
    pub enabled: bool,
}
```

**Invariants enforced by `Project::validate()`:**

1. `clip.start < clip.end` for every clip.
2. `clip.start >= 0` and `clip.end <= project.duration` (duration is recomputed lazily on first `to_json()` after edits).
3. `transform.x + transform.w <= 1.0 + eps` and similarly for y/h (clamp to range in the mutation API; the model stores the clamped value). `transform.z` is any `i32`; lower values are drawn first.
4. `SourceVideo` / `SourceAudio` clips live on the project's main video/audio track only.
5. `Text` clips have `text == Some(_)`. `Image` / `Video` clips have `source == Some(_)`.
6. `EffectInstance.params` is non-empty and every key is in the param schema for that `EffectKind`.

## 3. Rust API (data-oriented, exposed via wasm-bindgen)

```rust
#[wasm_bindgen]
impl Project {
    pub fn new(name: String, width: u32, height: u32) -> Project;

    // CRUD
    pub fn add_track(&mut self, kind: TrackKind) -> TrackId;
    pub fn remove_track(&mut self, id: TrackId) -> Result<(), JsError>;
    pub fn add_clip(&mut self, track_id: TrackId, clip: ClipInput) -> Result<ClipId, JsError>;
    pub fn update_clip(&mut self, id: ClipId, patch: ClipPatch) -> Result<(), JsError>;
    pub fn remove_clip(&mut self, track_id: TrackId, id: ClipId) -> Result<(), JsError>;
    pub fn move_clip(&mut self, id: ClipId, to_track: TrackId, new_start: f32) -> Result<(), JsError>;

    // Editing
    pub fn split_clip(&mut self, id: ClipId, at_time: f32) -> Result<ClipId, JsError>;
    // `merge_clips` requires all `ids` to be on the same track, sorted by `start`,
    // and contiguous (no gaps, no overlaps). Returns `Err` with a descriptive
    // JsError otherwise. The merged clip's `start` is `min(starts)` and `end` is
    // `max(ends)`. Effects, transform, opacity, kind, source, and text are taken
    // from the first clip in the input order. The returned `ClipId` is the new
    // merged clip's id.
    pub fn merge_clips(&mut self, track_id: TrackId, ids: Vec<ClipId>) -> Result<ClipId, JsError>;
    // `cut_range` removes the half-open time window `[start, end)` from each
    // clip in `selection`. For each affected clip, the portion before `start`
    // (if any) is kept, the portion after `end` (if any) is shifted left by
    // `(end - start)`, and the middle is removed. Clips not in `selection` are
    // unaffected regardless of whether they intersect the cut range.
    pub fn cut_range(&mut self, start: f32, end: f32, selection: Vec<(TrackId, ClipId)>) -> Result<(), JsError>;

    // Effects
    pub fn add_effect(&mut self, clip_id: ClipId, kind: EffectKind) -> Result<EffectId, JsError>;
    pub fn update_effect(&mut self, clip_id: ClipId, effect_id: EffectId, param: String, value: f32) -> Result<(), JsError>;
    pub fn remove_effect(&mut self, clip_id: ClipId, effect_id: EffectId) -> Result<(), JsError>;

    // Queries
    pub fn to_json(&self) -> JsValue;                           // for Vue to render
    pub fn layers_at(&self, time: f32) -> JsValue;              // LayerDescriptor[] for compositor
    pub fn validate(&self) -> Result<(), JsError>;             // invariants
}

#[wasm_bindgen]
pub fn apply_filter_pipeline(
    width: u32,
    height: u32,
    rgba_in: &[u8],
    rgba_out: &mut [u8],
    pipeline: JsValue,              // [{kind, params}, ...]
) -> Result<(), JsError>;
```

`pipeline` is a JS array. Each element has shape `{ kind: "grayscale" | "sepia" | ... , params: { name: number } }`. The function is pure, allocation-light, and reentrant. `apply_filter_pipeline` allocates no JS-side memory besides what `serde_wasm_bindgen` uses for parsing `pipeline`.

**Effect kinds shipped in this phase:**

| Kind       | Params                          | Notes |
|------------|---------------------------------|-------|
| Grayscale  | `intensity` (0..1)              | Luma-weighted blend with original |
| Sepia      | `intensity` (0..1)              | |
| Brightness | `level` (0.1..3.0)              | Multiply |
| Contrast   | `level` (0..2)                  | Around 0.5 |
| Saturate   | `level` (0..2)                  | 0 = grayscale, 1 = original |
| Invert     | (no params)                     | |
| Blur       | `radius` (1..16, integer)       | Box blur, separable, two-pass |
| ChromaKey  | `r,g,b` (0..1) `tolerance` (0..1) `softness` (0..1) | Sets alpha to 0 outside softness band |

New effect kinds are added by extending the `match` in `filter::run` and adding a unit test in `filter::tests`. There is no plugin mechanism in this phase.

## 4. Vue layer

### Pinia store (cache, not source of truth)

```js
state
  project: ProjectSnapshot  // set by to_json() pull
  ui: { selectedIds, currentTime, playing, isReady }
actions
  // each action: await editor.<method>(), then pull to_json, commit to state
  addClip, updateClip, moveClip, split, merge, cut, addEffect, updateEffect, removeEffect, ...
```

The store shape mirrors what components already consume (`tracks: [{ items: [...] }]`), so existing components continue to render. Source-video/source-audio placeholders are no longer inserted in the Vue layer — Rust inserts them automatically on `add_clip` of the imported source.

### `useEditor()` composable

```js
// video-editor/src/editor/index.js
import init, { Project, apply_filter_pipeline } from '../wasm/video_processor'

let project = null
let initPromise = null

export async function ensureInit() { /* boots WASM once */ }
export async function createProject({name, width, height}) { /* new Project(...) */ }
export function getProject() { return project }
export async function pullSnapshot() { return project.to_json() }
export async function layersAt(time) { return project.layers_at(time) }
export function applyFilter(w, h, inPx, outPx, pipeline) {
  return apply_filter_pipeline(w, h, inPx, outPx, pipeline)
}
```

### Compositor (`PreviewCanvas.vue`)

Replaces the rAF loop currently embedded in `VideoPreview.vue`. The existing overlays UI is preserved as the *interaction layer* above the canvas.

```
on every rAF:
  1. layers = editor.layersAt(currentTime)              // sorted by z, asc
  2. clear output canvas
  3. for each layer in layers:
     - if kind == 'video' or 'source-video':
         ensure off-screen <video> for src url is alive (LRU cache, max 6)
         sync video.currentTime to (currentTime - clip.start + clip.in_offset)
         drawImage(video) onto output at clip.transform
     - if kind == 'text':
         rasterize text to off-screen canvas via Canvas2D fillText
         drawImage(off-screen) onto output at clip.transform
     - if kind == 'image':
         ensure off-screen <img> for src is loaded (LRU cache, max 16)
         drawImage(img) onto output at clip.transform
     - apply opacity via globalAlpha
     - if layer has effects:
         a. extract ImageData (clip region only)
         b. apply_filter_pipeline(in, out, pipeline)
         c. putImageData back
  4. emit time-update / duration-change / playing-change as today
  // Audio for the source clip is played by the project's source <video> element
  // (current behavior, unchanged). The hidden source <video> drives audio out.
  // All other audio is Phase 3.
```

A layer with no effects skips the `getImageData`/`putImageData` round trip entirely. The compositor never calls into Rust for state — it only reads the descriptor list and the filter pipeline.

## 5. Crate layout

```
src/
  lib.rs                 // pub mod re-exports + #[wasm_bindgen(start)] init
  project.rs             // Project, Track, Clip, mutations, validation, to_json, layers_at
  model.rs               // pure types: TrackId, ClipId, EffectKind, Transform, ...
  effects.rs             // EffectKind enum, param schema, defaults
  filter.rs              // apply_filter_pipeline: pure RGBA in/out, single match dispatch
  filter/
    color.rs             // grayscale, sepia, brightness, contrast, saturate, invert
    blur.rs              // box blur, gaussian (separable)
    chroma.rs            // chroma key (color + tolerance + softness)
  text.rs                // placeholder for future text rasterization hook (no font engine)
  error.rs               // JsError mapping
  util.rs                // id generation, f32 clamping, time helpers
```

`lib.rs` keeps the existing `#[wasm_bindgen(start)]` panic hook. The `cdylib` crate type is preserved.

## 6. File changes

| File | Action |
|------|--------|
| `src/lib.rs` | Replaced with the module re-export + `init` only |
| `src/project.rs` | NEW |
| `src/model.rs` | NEW |
| `src/effects.rs` | NEW |
| `src/filter.rs` | NEW |
| `src/filter/color.rs` | NEW |
| `src/filter/blur.rs` | NEW |
| `src/filter/chroma.rs` | NEW |
| `src/text.rs` | NEW (placeholder) |
| `src/error.rs` | NEW |
| `src/util.rs` | NEW |
| `video-editor/src/editor/index.js` | NEW |
| `video-editor/src/editor/filterPresets.js` | NEW |
| `video-editor/src/components/PreviewCanvas.vue` | NEW |
| `video-editor/src/components/VideoEditor.vue` | Refactor: replace Pinia mutations with `useEditor()`; remove `ensureBaseClips` |
| `video-editor/src/components/Timeline.vue` | Unchanged behavior; small key/UX tweaks if needed |
| `video-editor/src/components/Inspector.vue` | Writes go through `useEditor()` |
| `video-editor/src/components/Controls.vue` | Add a small "Effects" popover with real preset menu |
| `video-editor/src/components/VideoPreview.vue` | Renamed to `PreviewCanvas.vue` (or kept as the interaction layer) |
| `video-editor/src/stores/project.js` | REPLACED — cache-only store |
| `video-editor/src/stores/counter.js` | Removed (unused) |
| `video-editor/src/wasm/*` | Regenerated by `build.sh` |
| `build.sh` | Unchanged |
| `Cargo.toml` | Add `serde` (with `derive`), `serde-wasm-bindgen` |

## 7. Test plan

| Layer | Type | What is covered |
|-------|------|-----------------|
| Rust model | `cargo test` unit | `split`, `merge`, `cut`, `move_clip`, validation, duration recompute, ID uniqueness |
| Rust filter | `cargo test` unit | per-effect invariants (`grayscale` → R=G=B; `invert` is involutive; `brightness(1)` is identity) |
| Rust filter | `cargo test` property | pipeline order: running `[A, B]` equals running `A` then `B` over the same buffer |
| Rust filter | Visual regression | 3 PNG fixtures (luma gradient, color bars, text mask) → expected PNGs diffed with `image::ImageBuffer::diff`, tolerance ≤ 2% perceptual |
| Rust pipeline | Snapshot | `apply_filter_pipeline` with a fixed pipeline → known output bytes |
| Vue (Vitest) | Unit | `useEditor` mocks the WASM module; verifies the store is updated after each action |
| Vue component | Vue Test Utils | Timeline renders tracks; Inspector patches update store; Controls fires `addFilter` |
| End-to-end | Dev-server smoke | `tests/smoke.mjs` loads the page, uploads a sample video, adds a clip, applies a filter, and asserts no console errors |

Visual regression uses a small corpus committed under `tests/fixtures/`. The script is `cargo test --test visual_regression`. PNGs are generated once with `BLESS=1 cargo test --test visual_regression` and reviewed in the PR.

## 8. Milestones

1. **M1 — Rust core model & API.** Types, mutations, validation, `to_json`, `layers_at`, all model unit tests green. `cargo build --target wasm32-unknown-unknown --release` succeeds; the existing dev server still runs.
2. **M2 — Rust filter pipeline.** `apply_filter_pipeline` with all eight effects listed in §3. Visual regression + property tests green.
3. **M3 — Vue bridge.** `useEditor`, Pinia cache store, all components refactored. Vitest unit tests green.
4. **M4 — Compositor rewrite.** `PreviewCanvas.vue` correctly composites sources, text, image, and applies effects at preview time. Manual smoke test against three demo videos.
5. **M5 — Hardening.** Error boundaries, dev-server smoke test, README updates, release build (`build.sh`) succeeds, dev server starts and renders a 5-second edit without console errors.

Each milestone produces a runnable editor at every step (no big-bang merge).

## 9. Non-goals (explicit YAGNI for this phase)

- No real video export. The export button remains a stub; Phase 3 builds WebCodecs + muxer.
- No audio mixing. Source audio is wired through in the simplest way; mixing is Phase 3.
- No transitions between clips.
- No animation/keyframes. `transform` is static per clip.
- No text shaping beyond browser default font (no HarfBuzz, no custom fonts in this phase). Text rasterization is done in JS via Canvas2D.
- No undo/redo. Added in a small follow-up spec.
- No project save/load. `localStorage` persistence of `to_json()` is added in a small follow-up spec.
- No mobile/touch support.

## 10. Risks and mitigations

- **Risk:** Rust <-> Vue per-frame call is slow.
  - **Mitigation:** measure with a simple perf counter in dev; if a layer's filter pipeline is empty, skip the round trip entirely (the JS compositor already draws the layer). Filter round trips only run for layers with effects.
- **Risk:** Off-screen `<video>` per clip uses memory.
  - **Mitigation:** LRU cache with a max of N=6 active videos; oldest unmounted on overflow.
- **Risk:** `wasm-bindgen` ergonomics around `BTreeMap<String, f32>`.
  - **Mitigation:** use a `Vec<JsParam>` shape for params on the wire; convert to/from `BTreeMap` only inside Rust.
- **Risk:** Vue's "ensure base clips" pattern is implicit. Removing it without equivalent Rust logic will leave the source video off-screen.
  - **Mitigation:** Rust inserts a `SourceVideo` clip on the first video track (or creates one) when a media file is imported, and a `SourceAudio` clip on the first audio track. This is covered by a model test.
- **Risk:** `serde-wasm-bindgen` deserialization of `pipeline` allocates per call.
  - **Mitigation:** the function is on a hot path only when a layer has effects; this is acceptable for Phase 1. Phase 2 (effects library) introduces a pre-parsed pipeline cache if profiling demands it.

## 11. Out-of-phase follow-ups (each becomes its own spec)

- A. **Export pipeline** — WebCodecs `VideoEncoder`, MP4 muxer, frame-by-frame composition using the same `layers_at()` query.
- B. **Effects library expansion** — LUTs, blend modes, vignette, grain, glitch.
- C. **Transitions** — crossfade, wipe, custom easing.
- D. **Animation & keyframes** — time-based property curves for transform and opacity.
- E. **Audio engine** — multi-track mixing, gain envelopes, fade, Web Audio routing.
- F. **Text engine** — HarfBuzz shaping, custom font loading, outline, shadow.
- G. **Undo/redo** — command pattern over the Rust mutation API.
- H. **Project save/load** — `localStorage` + JSON serialization (initial version of persistence).

Each of these is small enough to design and plan in one spec, and each builds on the foundation shipped here.
