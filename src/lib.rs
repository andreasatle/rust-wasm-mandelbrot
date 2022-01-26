//! snake_game is the engine of a Snake Game for the browser.

/// Activate wasm_bindgen to be able to compile to wasm.
use wasm_bindgen::prelude::*;

/// Replace the default allocator with wee_alloc.
/// This is suitable when compiling to wasm.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen(module = "/www/utils/utils.ts")]
// extern "C" {
    // fn output_js(msg: String);
// }

/// A Point for f64.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
struct PointF64 {
    pub x: f64,
    pub y: f64,
}

/// A Point for usize.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
struct PointUsize {
    x: usize,
    y: usize,
}

#[wasm_bindgen]
pub struct Mandelbrot {
    z0: PointF64,
    n: PointUsize,
    d: PointF64,
    max_iter: u32,
    img: Vec<u32>,
    colormap: Vec<u32>,
}

#[wasm_bindgen]
impl Mandelbrot {
    /// Constructor takes too many parameters.
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64, nx: usize, ny: usize, max_iter: u32, red: u8, green: u8, blue: u8) -> Mandelbrot {
        let mut mandel = Mandelbrot {
            z0: PointF64{x: x0, y: y0},
            n: PointUsize{x: nx, y: ny},
            d: PointF64{x: (x1-x0) / nx as f64, y: (y1-y0) / ny as f64},
            max_iter,
            img: Vec::with_capacity(nx*ny),
            colormap: Vec::with_capacity(max_iter as usize),
        };
        mandel.set_colormap(red, green, blue);
        mandel
    }

    pub fn set_colormap(&mut self, red: u8, green: u8, blue: u8) {
        // Shift rgb values to their place in rgba, leaving a empty.
        let rgba0 = ((red as u32) << 24) + ((green as u32) << 16) + ((blue as u32) << 8);
        let warp = |x: f64| x.cbrt();
        for idx in 0..self.max_iter {
            let alpha = idx as f64/self.max_iter as f64;
            self.colormap.push(rgba0 + (warp(alpha)*255.0) as u32);
        }

    }
    pub fn reset(&mut self, rx0: f64, ry0: f64, dx: f64, dy: f64) {

        let dxy = if dx.abs() > dy.abs() {dx} else {dy};

        self.z0.x += rx0*self.d.x*self.n.x as f64;
        self.z0.y += ry0*self.d.y*self.n.y as f64;
        self.d.x = dxy*self.d.x;
        self.d.y = dxy*self.d.y;
    }

    fn get_coord(&self, idx: &PointUsize) -> PointF64 {
        PointF64 {
            x: self.z0.x+(idx.x as f64+0.5)*self.d.x,
            y: self.z0.y+(idx.y as f64+0.5)*self.d.y,
        }
    }

    fn count_iter_for_idx(&self, idx: &PointUsize) -> u32 {
        let c = self.get_coord(&idx);
        let mut z = PointF64{x:0.0, y:0.0};
        for iter in 0..self.max_iter {
            // Check |z| >= 2 for divergence.
            if z.x*z.x + z.y*z.y >= 2.0 {
                return iter
            }
            // Update z <- z*z + c
            let zx = z.x*z.x - z.y*z.y + c.x;
            z.y = 2.0*z.x*z.y + c.y;
            z.x = zx;
        }
        // Return 0 when max-iter reached.
        0
    }

    /// Count the #iterations.
    fn count_iterations(&mut self) {
        self.img.clear();
        let mut idx: PointUsize = PointUsize{x:0,y:0};
        for iy in 0..self.n.y {
            idx.y = iy;
            for ix in 0..self.n.x {
                idx.x = ix;
                self.img.push(self.count_iter_for_idx(&idx));
            }
        }
    }

    /// Change representation of image from #iterations to a rgba-color.
    fn iterations_to_color(&mut self) {
        for idx in 0..self.n.x*self.n.y {
            self.img[idx] = self.colormap[self.img[idx] as usize];
        }
    }

    /// Return the pointer to the image.
    pub fn get_image(&self) -> *const u32 {
        self.img.as_ptr()
    }

    /// zoom in
    pub fn zoom(&mut self, x0: f64, y0: f64, dx: f64, dy: f64) {
        self.reset(x0, y0, dx, dy);
        self.count_iterations();
        self.iterations_to_color();
    }
    
}


