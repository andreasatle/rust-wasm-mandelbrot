/// mandelbrot is the engine of a Mandelbrot Image Generator for the browser.
/// It is used with typescript to setup a minimal web-application, where it is
/// possible to zoom in the image, which is automatically regenerated.

/// Activate wasm_bindgen to be able to compile to wasm.
use wasm_bindgen::prelude::*;

/// Replace the default allocator with wee_alloc.
/// This is suitable when compiling to wasm.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Import utility-functions from typescript.
#[wasm_bindgen(module = "/www/utils/utils.ts")]
extern "C" {
    /// Write a message to the console in the web-browser.
    /// 
    /// It's convenient to use the ```format!``` macro
    /// to build a String.
    /// 
    /// * ```msg```: Message to be written.
    fn output_js(msg: String);
}

/// A Point for f64.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
#[derive(Clone,Copy)]
struct PointF64 {
    x: f64,
    y: f64,
}

/// A Point for usize.
/// Remark: Generics doesn't seem to work with WASM.
#[wasm_bindgen]
#[derive(Clone,Copy)]
struct PointUsize {
    x: usize,
    y: usize,
}

struct MetaData {
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

}

impl MetaData {
    /// Rescale the box that constitutes the Mandelbrot image.
    fn rescale_problem(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        // Make sure that x1 > x0, and y1 > y0.
        let (x0,x1) = if x1 > x0 {(x0,x1)} else {(x1,x0)};
        let (y0,y1) = if y1 > y0 {(y0,y1)} else {(y1,y0)};

        // Let dxy be the largest side in the rectangle.
        let dxy = if x1-x0 > y1-y0 {x1-x0} else {y1-y0};

        self.z0.x += x0*self.d.x*self.n.x as f64;
        self.z0.y += y0*self.d.y*self.n.y as f64;
        self.d.x = dxy*self.d.x;
        self.d.y = self.d.x;
    }

    /// Compute the escape iteration for one point c.
    /// 0 is returned when the maximum number of iterations are reached.
    fn count_iter_for_index(&self, i: usize) -> usize {
        let c = self.get_coord(i);
        let mut z = PointF64{x:0.0, y:0.0};
        for iter in 0..self.max_iter {
            // Check |z|^2 >= 4 for escape-iteration.
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
    /// Get the coordinate for a multiple-index in the image.
    fn get_coord(&self, i: usize) -> PointF64 {
        PointF64 {
            x: self.z0.x + ((i%self.n.x) as f64 + 0.5) * self.d.x,
            y: self.z0.y + ((i/self.n.x) as f64 + 0.5) * self.d.y,
        }
    }

}

/// Contains all necessary info about the Mandelbrot image.
#[wasm_bindgen]
pub struct Mandelbrot {

    meta: MetaData,

    /// Work vector of full image size.
    work: Vec<usize>,

    /// Image represented with u8.
    img: Vec<u8>,

    /// Mapping from escape-iteration to interpolation-weight for the color.
    iterations: Vec<usize>,
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

        Mandelbrot {
            meta: MetaData {
                z0: PointF64{x: x0, y: y0},
                n: PointUsize{x: nx, y: ny},
                d: PointF64{x: (x1-x0) / nx as f64, y: (y1-y0) / ny as f64},
                max_iter,
                n_colors,
                red,
                green,
                blue,
            },
            work: vec![0;nx*ny],
            img: vec![0;4*nx*ny],
            iterations: vec![0;max_iter],
        }
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
        self.meta.rescale_problem(x0, y0, x1, y1);

        // Count the escape iterations.
        self.count_iterations();

        // Compute statistics for the different escape iterations.
        self.iteration_frequency();

        // Take cumulative sum of the frequencies of the escape iterations.
        self.frequency_cumsum();

        // Bin s.t. the cumulative sum becomes linear.
        // This defines the iterations for each escape iteration.
        self.iteration_binner();

        // Fill the image with the correct RGBA-color.
        self.iterations_to_color();
    }
    
    /// Count the escape iterations for all indices in the image.
    fn count_iterations(&mut self) {
        // I want the iteration in this form, so I can use rayon.
        for (i,v) in self.work.iter_mut().enumerate() {
            *v = self.meta.count_iter_for_index(i);
        }
    }

    /// Change representation of image from #iterations to a rgba-color.
    fn iterations_to_color(&mut self) {

        for (i,w) in self.work.iter().enumerate() {
            let i4 = i << 2;
            self.img[i4] = ((self.meta.red as usize*self.iterations[*w])/self.meta.n_colors) as u8;
            self.img[i4+1] = ((self.meta.green as usize*self.iterations[*w])/self.meta.n_colors) as u8;
            self.img[i4+2] = ((self.meta.blue as usize*self.iterations[*w])/self.meta.n_colors) as u8;
            self.img[i4+3] = 255;
        }
    }
    
    /// Count the frequency (or occurance) of each escape iteration.
    fn iteration_frequency(&mut self) {

        // Reset the counters.
        for v in self.iterations.iter_mut() {
            *v = 0;
        }
        
        // Count the frequency of the different iterations.
        for (i,_) in self.work.iter().enumerate() {
            self.iterations[self.work[i]] += 1;
        }
    }

    /// Take the cumulative sum, except for the first entry.
    fn frequency_cumsum(&mut self) {
        // Skip the count of the actual Mandelbrot Set.
        self.iterations[0] = 0;

        // Cumulative sum of the iteration frequencies
        for i in 1..self.meta.max_iter {
            self.iterations[i] += self.iterations[i-1];
        }
    }

    /// Bin the different number of iterations according to their frequencies.
    fn iteration_binner(&mut self) {
        let threshold = self.iterations[self.meta.max_iter-1] / (self.meta.n_colors-1);
        let mut bin = 0;
        // This is a sequential process.
        for i in 1..self.meta.max_iter {
            if self.iterations[i] > threshold*bin {
                bin += 1;
            }
            self.iterations[i] = bin;
        }
    }
}


