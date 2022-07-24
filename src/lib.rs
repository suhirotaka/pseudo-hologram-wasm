mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, ImageData};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Define constants
// TODO: Dynamic setting of hologram size
//       This needs to fix large memory allocation to stack
const HOLOGRAM_SIZE: u32 = 512;
const HOLOGRAM_FPS: u32 = 60;

// Import JS APIs
#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
    fn clearInterval(id: i32);
}

#[wasm_bindgen]
pub struct IntervalHandle {
    pub interval_id: i32,
    _closure: Closure<dyn FnMut()>,
}

impl Drop for IntervalHandle {
    fn drop(&mut self) {
        clearInterval(self.interval_id);
    }
}

#[wasm_bindgen]
pub struct PseudoHologram {}

#[wasm_bindgen]
impl PseudoHologram {
    pub fn draw(hologram_canvas_id: &str, orignal_video_id: &str) -> IntervalHandle {
        utils::set_panic_hook();
        Processor::start(hologram_canvas_id, orignal_video_id)
    }
}

struct Processor {
    video: HtmlVideoElement,
    video_width: u32,
    video_height: u32,
    canvas: HtmlCanvasElement,
}

impl Processor {
    fn start(hologram_canvas_id: &str, orignal_video_id: &str) -> IntervalHandle {
        let document = web_sys::window().unwrap().document().unwrap();

        let canvas = document
            .get_element_by_id(hologram_canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let video = document
            .get_element_by_id(orignal_video_id)
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap();

        canvas.set_width(HOLOGRAM_SIZE);
        canvas.set_height(HOLOGRAM_SIZE);

        // Resize original video to fit HOLOGRAM_SIZE
        let video_height = (HOLOGRAM_SIZE as f32
            / (2.0 + video.client_width() as f32 / video.client_height() as f32))
            as u32;
        let video_width = HOLOGRAM_SIZE - 2 * video_height;
        video.set_width(video_width);
        video.set_height(video_height);

        let processor = Processor {
            video,
            canvas,
            video_width,
            video_height,
        };

        let closure = Closure::new(move || {
            processor.compute_frame();
        });
        let interval_id = setInterval(&closure, 1000 / HOLOGRAM_FPS);

        let interval_handle = IntervalHandle {
            interval_id,
            _closure: closure,
        };

        interval_handle
    }

    fn compute_frame(&self) {
        if self.video.paused() || self.video.ended() {
            return;
        }

        let context = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        let video_width = self.video_width;
        let video_height = self.video_height;

        context
            .draw_image_with_html_video_element_and_dw_and_dh(
                &self.video,
                0.0,
                0.0,
                video_width as f64,
                video_height as f64,
            )
            .unwrap();

        let video_frame = context
            .get_image_data(0.0, 0.0, video_width as f64, video_height as f64)
            .unwrap()
            .data();

        let mut pixels: [u8; (HOLOGRAM_SIZE * HOLOGRAM_SIZE * 4) as usize] =
            [0; (HOLOGRAM_SIZE * HOLOGRAM_SIZE * 4) as usize];
        for i in 0..(pixels.len() / 4) {
            let hologram_x = i as u32 % HOLOGRAM_SIZE;
            let hologram_y = i as u32 / HOLOGRAM_SIZE;

            // Fill black pixels
            if (hologram_x < video_height && hologram_y < video_height)
                || (hologram_x < video_height && hologram_y >= video_height + video_width)
                || (hologram_x >= video_width + video_height && hologram_y < video_height)
                || (hologram_x >= video_width + video_height
                    && hologram_y >= video_height + video_width)
                || (hologram_x >= video_height
                    && hologram_x < video_width + video_height
                    && hologram_y >= video_height
                    && hologram_y < video_width + video_height)
            {
                Processor::set_pixel(&mut pixels, i, [0, 0, 0, 255]);
            }

            // Fill top video pixels
            if hologram_x >= video_height
                && (hologram_x < video_width + video_height)
                && hologram_y < video_height
            {
                let frame_x = hologram_x - video_height;
                let frame_y = hologram_y;

                Processor::set_pixel(
                    &mut pixels,
                    i,
                    [
                        video_frame[((frame_x + video_width * frame_y) * 4) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 1) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 2) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 3) as usize],
                    ],
                );
            }

            // Fill left video pixels
            if hologram_x < video_height
                && hologram_y >= video_height
                && (hologram_y < video_width + video_height)
            {
                let frame_x = video_width + video_height - hologram_y - 1;
                let frame_y = hologram_x;

                Processor::set_pixel(
                    &mut pixels,
                    i,
                    [
                        video_frame[((frame_x + video_width * frame_y) * 4) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 1) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 2) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 3) as usize],
                    ],
                );
            }

            // Fill bottom video pixels
            if hologram_x >= video_height
                && hologram_x < video_width + video_height
                && hologram_y >= video_width + video_height
            {
                let frame_x = video_width + video_height - hologram_x - 1;
                let frame_y = video_width + video_height * 2 - hologram_y - 1;

                Processor::set_pixel(
                    &mut pixels,
                    i,
                    [
                        video_frame[((frame_x + video_width * frame_y) * 4) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 1) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 2) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 3) as usize],
                    ],
                );
            }

            // Fill right video pixels
            if hologram_x >= video_width + video_height
                && hologram_y >= video_height
                && (hologram_y < video_width + video_height)
            {
                let frame_x = hologram_y - video_height;
                let frame_y = video_width + video_height * 2 - hologram_x - 1;

                Processor::set_pixel(
                    &mut pixels,
                    i,
                    [
                        video_frame[((frame_x + video_width * frame_y) * 4) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 1) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 2) as usize],
                        video_frame[((frame_x + video_width * frame_y) * 4 + 3) as usize],
                    ],
                );
            }
        }

        context
            .put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(
                &ImageData::new_with_u8_clamped_array(Clamped(&pixels), HOLOGRAM_SIZE).unwrap(),
                0.0,
                0.0,
                0.0,
                0.0,
                HOLOGRAM_SIZE as f64,
                HOLOGRAM_SIZE as f64,
            )
            .unwrap();
    }

    fn set_pixel(vec: &mut [u8], pixel: usize, color: [u8; 4]) {
        vec[pixel * 4] = color[0];
        vec[pixel * 4 + 1] = color[1];
        vec[pixel * 4 + 2] = color[2];
        vec[pixel * 4 + 3] = color[3];
    }
}
