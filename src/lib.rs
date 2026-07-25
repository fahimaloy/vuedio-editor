use js_sys::{Function, Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::HtmlVideoElement;

use image::{Rgba, RgbaImage};

/// Better panic messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct VideoProcessor {
    width: u32,
    height: u32,
    frame_data: Vec<u8>, // RGBA8, len = width * height * 4
}

#[wasm_bindgen]
impl VideoProcessor {
    /// Construct a processor with an empty RGBA buffer.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Result<VideoProcessor, JsValue> {
        if width == 0 || height == 0 {
            return Err(js_err("width and height must be > 0"));
        }
        let len = width
            .checked_mul(height)
            .and_then(|px| px.checked_mul(4))
            .ok_or_else(|| js_err("dimension overflow"))? as usize;

        Ok(VideoProcessor {
            width,
            height,
            frame_data: vec![0; len],
        })
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Copy image bytes into the internal buffer. `pixels.len()` must equal width*height*4.
    pub fn load_frame(&mut self, pixels: &[u8]) -> Result<(), JsValue> {
        if pixels.len() != self.frame_data.len() {
            return Err(js_err(&format!(
                "expected RGBA buffer of length {}, got {}",
                self.frame_data.len(),
                pixels.len()
            )));
        }
        self.frame_data.copy_from_slice(pixels);
        Ok(())
    }

    /// Return a JS copy of the RGBA buffer (safe default).
    pub fn get_frame(&self) -> Uint8Array {
        Uint8Array::from(self.frame_data.as_slice())
    }
    /// Pointer to the start of the RGBA buffer (for zero-copy access from JS).
    /// Pair with `frame_len()`, and remember to re-fetch after any mutation.
    pub fn frame_ptr(&self) -> *const u8 {
        self.frame_data.as_ptr()
    }

    /// Length in bytes of the RGBA buffer.
    pub fn frame_len(&self) -> usize {
        self.frame_data.len()
    }

    /// Convert the current frame to grayscale (in-place).
    pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
        let mut img = to_rgba_image(self.width, self.height, &self.frame_data)?;
        for p in img.pixels_mut() {
            // Luma approximation for sRGB
            let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
                .round()
                .clamp(0.0, 255.0) as u8;
            *p = Rgba([l, l, l, p[3]]);
        }
        self.frame_data = img.into_raw();
        Ok(())
    }

    /// Sepia (in-place).
    pub fn apply_sepia(&mut self) -> Result<(), JsValue> {
        let mut img = to_rgba(self.width, self.height, &self.frame_data)?;
        for p in img.pixels_mut() {
            let (r, g, b, a) = (p[0] as f32, p[1] as f32, p[2] as f32, p[3]);
            let tr = (0.393 * r + 0.769 * g + 0.189 * b).clamp(0.0, 255.0) as u8;
            let tg = (0.349 * r + 0.686 * g + 0.168 * b).clamp(0.0, 255.0) as u8;
            let tb = (0.272 * r + 0.534 * g + 0.131 * b).clamp(0.0, 255.0) as u8;
            *p = Rgba([tr, tg, tb, a]);
        }
        self.frame_data = img.into_raw();
        Ok(())
    }

    /// Brightness multiply (in-place). 1.0 = unchanged.
    pub fn apply_brightness(&mut self, factor: f32) -> Result<(), JsValue> {
        if !factor.is_finite() || factor < 0.0 {
            return Err(js_err(
                "brightness factor must be a finite, non-negative number",
            ));
        }
        let mut img = to_rgba(self.width, self.height, &self.frame_data)?;
        for p in img.pixels_mut() {
            p[0] = (p[0] as f32 * factor).clamp(0.0, 255.0) as u8;
            p[1] = (p[1] as f32 * factor).clamp(0.0, 255.0) as u8;
            p[2] = (p[2] as f32 * factor).clamp(0.0, 255.0) as u8;
        }
        self.frame_data = img.into_raw();
        Ok(())
    }

    /// Contrast adjustment (in-place). 1.0 = unchanged, >1.0 = more contrast, <1.0 = less contrast.
    pub fn apply_contrast(&mut self, factor: f32) -> Result<(), JsValue> {
        if !factor.is_finite() {
            return Err(js_err("contrast factor must be finite"));
        }
        let mut img = to_rgba(self.width, self.height, &self.frame_data)?;
        let contrast = factor.clamp(0.0, 3.0);
        for p in img.pixels_mut() {
            let rf = (p[0] as f32 - 128.0) * contrast + 128.0;
            let gf = (p[1] as f32 - 128.0) * contrast + 128.0;
            let bf = (p[2] as f32 - 128.0) * contrast + 128.0;
            p[0] = rf.clamp(0.0, 255.0) as u8;
            p[1] = gf.clamp(0.0, 255.0) as u8;
            p[2] = bf.clamp(0.0, 255.0) as u8;
        }
        self.frame_data = img.into_raw();
        Ok(())
    }

    /// Saturation adjustment (in-place). 1.0 = unchanged, >1.0 = more saturated.
    pub fn apply_saturation(&mut self, factor: f32) -> Result<(), JsValue> {
        if !factor.is_finite() || factor < 0.0 {
            return Err(js_err(
                "saturation factor must be a finite, non-negative number",
            ));
        }
        let mut img = to_rgba(self.width, self.height, &self.frame_data)?;
        for p in img.pixels_mut() {
            let l = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            p[0] = ((p[0] as f32 - l) * factor + l).clamp(0.0, 255.0) as u8;
            p[1] = ((p[1] as f32 - l) * factor + l).clamp(0.0, 255.0) as u8;
            p[2] = ((p[2] as f32 - l) * factor + l).clamp(0.0, 255.0) as u8;
        }
        self.frame_data = img.into_raw();
        Ok(())
    }

    /// Simple box blur (in-place). radius = blur strength.
    pub fn apply_blur(&mut self, radius: u32) -> Result<(), JsValue> {
        let r = (radius as usize).min(20);
        if r == 0 {
            return Ok(());
        }
        let mut img = to_rgba(self.width, self.height, &self.frame_data)?;
        let (w, h) = (img.width() as usize, img.height() as usize);
        let mut blurred = img.clone();

        for y in r..(h - r) {
            for x in r..(w - r) {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                for dy in 0..=r * 2 {
                    for dx in 0..=r * 2 {
                        let p = img.get_pixel((x + dx - r) as u32, (y + dy - r) as u32);
                        r_sum += p[0] as u32;
                        g_sum += p[1] as u32;
                        b_sum += p[2] as u32;
                    }
                }
                let count = ((r * 2 + 1) * (r * 2 + 1)) as u32;
                let p = blurred.get_pixel_mut(x as u32, y as u32);
                p[0] = (r_sum / count) as u8;
                p[1] = (g_sum / count) as u8;
                p[2] = (b_sum / count) as u8;
            }
        }
        self.frame_data = blurred.into_raw();
        Ok(())
    }
}

/// Placeholder for an async export pipeline (wire to your real implementation).
#[wasm_bindgen]
pub fn export_video(_video: HtmlVideoElement, progress_callback: &Function) -> js_sys::Promise {
    // Example: call progress once at 100%
    let _ = progress_callback.call1(&JsValue::NULL, &JsValue::from_f64(1.0));
    js_sys::Promise::resolve(&JsValue::UNDEFINED)
}

/* -------- helpers -------- */

fn js_err(msg: &str) -> JsValue {
    JsValue::from_str(msg)
}

fn to_rgba_image(width: u32, height: u32, buf: &[u8]) -> Result<RgbaImage, JsValue> {
    RgbaImage::from_raw(width, height, buf.to_vec())
        .ok_or_else(|| js_err("failed to create image from raw data"))
}
fn to_rgba(w: u32, h: u32, buf: &[u8]) -> Result<RgbaImage, JsValue> {
    RgbaImage::from_raw(w, h, buf.to_vec())
        .ok_or_else(|| js_err("failed to create image from raw data"))
}
