/// mandelbrot is the engine of a Mandelbrot Image Generator for the browser.
/// It is used with typescript to setup a minimal web-application, where it is
/// possible to zoom in the image, which is automatically regenerated.

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

/// Contains all necessary info about the Mandelbrot image.
#[wasm_bindgen]
pub struct Mandelbrot {
    /// Corner Point of image.
    z0: PointF64,

    /// Number of Points in each direction.
    n: PointUsize,

    /// Voxel size.
    d: PointF64,

    /// Maximum number of iterations until Mandebrot Set is assumed.
    max_iter: usize,
    
    /// Number of colors in image.
    n_colors: usize,

    /// Red RGB-value.
    red: u8,

    /// Green RGB-value.
    green: u8,

    /// Blue RGB-value.
    blue: u8,

    /// Work vector of full image size.
    work: Vec<usize>,

    /// Image represented with u8.
    img: Vec<u8>,

    /// Mapping from escape-iteration to interpolation-weight for the color.
    colormap: Vec<usize>,
}

#[wasm_bindgen]
impl Mandelbrot {
    /// Constructor initializes the parameters to the most scaled out image position.
    /// 
    /// * `x0`: x-coordinate of upper left corner of the image.
    /// * `y0`: y-coordinate of upper left corner of the image.
    /// * `x1`: x-coordinate of lower right corner of the image.
    /// * `y1`: y-coordinate of lower right corner of the image.
    /// * `nx`: Number of pixels in the x-direction.
    /// * `ny`: Number of pixels in the y-direction.
    /// * `max_iter`: Maximum number of iterations before concluded that point is in Mandelbrot Set.
    /// * `n_colors`: Number of colors in the rendered image.
    /// * `red`: Red RGB-value.
    /// * `green`: Green RGB-value.
    /// * `blue`: Blue RGB-value.
    pub fn new(
        x0: f64, y0: f64, x1: f64, y1: f64,
        nx: usize, ny: usize, max_iter: usize, n_colors: usize,
        red: u8, green: u8, blue: u8
    ) -> Mandelbrot {

        let mandel = Mandelbrot {
            z0: PointF64{x: x0, y: y0},
            n: PointUsize{x: nx, y: ny},
            d: PointF64{x: (x1-x0) / nx as f64, y: (y1-y0) / ny as f64},
            max_iter,
            n_colors,
            red,
            green,
            blue,
            work: Vec::with_capacity(nx*ny),
            img: Vec::with_capacity(4*nx*ny),
            colormap: Vec::with_capacity(max_iter),
        };
        mandel
    }

    /// Return the pointer to the image.
    /// This is used by typescript to get access to the memory
    /// for the image.
    pub fn get_image(&self) -> *const u8 {
        self.img.as_ptr()
    }

    /// Update image and compute a new image.
    /// 
    /// The input arguments are in "relative" coordinates in range [0,1],
    /// to avoid cancellation issues. I don't think it matters, but still...
    /// 
    /// * `x0`: Relative x-coordinate to the first corner in a rectangle.
    /// * `y0`: Relative y-coordinate to the first corner in a rectangle.
    /// * `x1`: Relative x-coordinate to the opposite corner in a rectangle.
    /// * `y1`: Relative y-coordinate to the opposite corner in a rectangle.
    pub fn update_image(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {

        // Setup the coordinates for the new image, using relative coordinates.
        self.rescale_problem(x0, y0, x1, y1);

        // Count the escape iterations.
        self.count_iterations();

        // Compute statistics for the different escape iterations.
        self.iteration_frequency();

        // Take cumulative sum of the frequencies of the escape iterations.
        self.frequency_cumsum();

        // Bin s.t. the cumulative sum becomes linear.
        // This defines the colormap for each escape iteration.
        self.iteration_binner();

        // Fill the image with the correct RGBA-color.
        self.iterations_to_color();
    }

    /// Rescale the box that constitutes the Mandelbrot image.
    fn rescale_problem(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {

        let mut rx0 = x0;
        let mut rx1 = x1;
        let mut ry0 = y0;
        let mut ry1 = y1;

        // Swap x if wrong orientation
        if rx0 > rx1 {
            rx0 = x1;
            rx1 = x0;
        }
        
        // Swap y if wrong orientation
        if ry0 > ry1 {
            ry0 = y1;
            ry1 = y0;
        }

        let dxy = if rx1-rx0 > ry1-ry0 {rx1-rx0} else {ry1-ry0};

        self.z0.x += rx0*self.d.x*self.n.x as f64;
        self.z0.y += ry0*self.d.y*self.n.y as f64;
        self.d.x = dxy*self.d.x;
        self.d.y = self.d.x;
    }

    /// Get the coordinate for a multiple-index in the image.
    fn get_coord(&self, i: &PointUsize) -> PointF64 {
        PointF64 {
            x: self.z0.x+(i.x as f64+0.5)*self.d.x,
            y: self.z0.y+(i.y as f64+0.5)*self.d.y,
        }
    }

    /// Compute the escape iteration for one index.
    /// 0 is returned when the maximum number of iterations are reached.
    fn count_iter_for_index(&self, i: &PointUsize) -> usize {
        let c = self.get_coord(&i);
        let mut z = PointF64{x:0.0, y:0.0};
        for iter in 0..self.max_iter {
            // Check |z| >= 2 for divergence.
            if z.x*z.x + z.y*z.y >= 4.0 {
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

    /// Count the escape iterations for all indices in the image.
    fn count_iterations(&mut self) {
        self.work.clear();
        let mut i: PointUsize = PointUsize{x:0,y:0};
        for iy in 0..self.n.y {
            i.y = iy;
            for ix in 0..self.n.x {
                i.x = ix;
                self.work.push(self.count_iter_for_index(&i));
            }
        }
    }

    /// Change representation of image from #iterations to a rgba-color.
    fn iterations_to_color(&mut self) {
        self.img.clear();
        self.img.resize(self.img.capacity(), 255);

        for i in 0..self.n.x*self.n.y {
            let i4 = i << 2;
            self.img[i4] = (self.red as usize*self.colormap[self.work[i]]/self.n_colors) as u8;
            self.img[i4+1] = (self.green as usize*self.colormap[self.work[i]]/self.n_colors) as u8;
            self.img[i4+2] = (self.blue as usize*self.colormap[self.work[i]]/self.n_colors) as u8;
        }
    }
    
    /// Count the frequency (or occurance) of each escape iteration.
    fn iteration_frequency(&mut self) {
        // Initialize the array to zero.
        self.colormap.clear();
        self.colormap.resize(self.colormap.capacity(),0);

        // Count the frequency of the different iterations.
        for i in 0..self.work.len() {
            self.colormap[self.work[i]] += 1;
        }
    }

    /// Take the cumulative sum, except for the first entry.
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


