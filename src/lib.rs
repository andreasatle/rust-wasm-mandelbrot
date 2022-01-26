//! snake_game is the engine of a Snake Game for the browser.

/// Activate wasm_bindgen to be able to compile to wasm.
use wasm_bindgen::prelude::*;

/// Replace the default allocator with wee_alloc.
/// This is suitable when compiling to wasm.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "/www/utils/utils.ts")]
extern "C" {
    fn output_js(msg: String);
}

/// A Point for f64.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
struct PointF64 {
    x: f64,
    y: f64,
}

/// A Point for usize.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
struct PointUsize {
    x: usize,
    y: usize,
}

/// Contains all info about the Mandelbrot image.
#[wasm_bindgen]
pub struct Mandelbrot {
    z0: PointF64,
    n: PointUsize,
    d: PointF64,
    max_iter: usize,
    n_colors: usize,
    red: u8,
    green: u8,
    blue: u8,
    img2: Vec<usize>,
    img: Vec<u8>,
    colormap: Vec<usize>,
}

#[wasm_bindgen]
impl Mandelbrot {
    /// Constructor initializes the parameters to the most scaled out image position.
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64, nx: usize, ny: usize, max_iter: usize, n_colors: usize, red: u8, green: u8, blue: u8) -> Mandelbrot {
        let mandel = Mandelbrot {
            z0: PointF64{x: x0, y: y0},
            n: PointUsize{x: nx, y: ny},
            d: PointF64{x: (x1-x0) / nx as f64, y: (y1-y0) / ny as f64},
            max_iter,
            n_colors,
            red,
            green,
            blue,
            img2: Vec::with_capacity(nx*ny),
            img: Vec::with_capacity(4*nx*ny),
            colormap: Vec::with_capacity(max_iter),
        };
        mandel
    }

    pub fn rescale_problem(&mut self, rx0: f64, ry0: f64, dx: f64, dy: f64) {

        let dxy = if dx.abs() > dy.abs() {dx} else {dy};

        self.z0.x += rx0*self.d.x*self.n.x as f64;
        self.z0.y += ry0*self.d.y*self.n.y as f64;
        self.d.x = dxy*self.d.x;
        self.d.y = dxy*self.d.y;
    }

    fn get_coord(&self, i: &PointUsize) -> PointF64 {
        PointF64 {
            x: self.z0.x+(i.x as f64+0.5)*self.d.x,
            y: self.z0.y+(i.y as f64+0.5)*self.d.y,
        }
    }

    fn count_iter_for_index(&self, i: &PointUsize) -> usize {
        let c = self.get_coord(&i);
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
        self.img2.clear();
        let mut i: PointUsize = PointUsize{x:0,y:0};
        for iy in 0..self.n.y {
            i.y = iy;
            for ix in 0..self.n.x {
                i.x = ix;
                self.img2.push(self.count_iter_for_index(&i));
            }
        }
    }

    /// Change representation of image from #iterations to a rgba-color.
    fn iterations_to_color(&mut self) {
        self.img.clear();
        self.img.resize(self.img.capacity(), 255);

        for i in 0..self.n.x*self.n.y {
            let i4 = i << 2;
            self.img[i4] = (self.red as usize*self.colormap[self.img2[i]]/self.n_colors) as u8;
            self.img[i4+1] = (self.green as usize*self.colormap[self.img2[i]]/self.n_colors) as u8;
            self.img[i4+2] = (self.blue as usize*self.colormap[self.img2[i]]/self.n_colors) as u8;
        }
    }

    /// Return the pointer to the image.
    pub fn get_image(&self) -> *const u8 {
        self.img.as_ptr()
    }

    /// update_image and prepare a new image.
    pub fn update_image(&mut self, x0: f64, y0: f64, dx: f64, dy: f64) {
        self.rescale_problem(x0, y0, dx, dy);
        self.count_iterations();
        self.iteration_frequency();
        self.frequency_cumsum();
        self.iteration_binner();
        self.iterations_to_color();
    }
    
    fn iteration_frequency(&mut self) {
        // Initialize the array to zero.
        self.colormap.clear();
        self.colormap.resize(self.colormap.capacity(),0);

        // Count the frequency of the different iterations.
        for i in 0..self.img2.len() {
            self.colormap[self.img2[i]] += 1;
        }
    }

    fn frequency_cumsum(&mut self) {
        // Skip the count of the actual Mandelbrot Set.
        self.colormap[0] = 0;

        // Cumulative sum of the iteration frequencies
        for i in 1..self.max_iter {
            self.colormap[i] += self.colormap[i-1];
        }
    }

    /// Bin the different number of iterations according to their frequencies.
    fn iteration_binner(&mut self) {
        let threshold = self.colormap[self.max_iter-1] / (self.n_colors-1);
        let mut bin = 0;
        for i in 1..self.max_iter {
            if self.colormap[i] > threshold*bin {
                bin += 1;
            }
            self.colormap[i] = bin;
        }
    }
}


