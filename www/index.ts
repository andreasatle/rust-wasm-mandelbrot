// This is the frontend (typescript) part of the project, which render a
// zoomable Mandelbrot image.
// This code interacts with Rust-wasm to get updated images.

// Import rust code.
import init, { Mandelbrot} from "mandelbrot";

// Setup the canvas from HTML.
const canvas = document.getElementById("mandelbrot-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");

// Set the size of the canvas and the final image.
canvas.height = 2048;
canvas.width = 2048;

const CORNER_X0 = -2.0;
const CORNER_Y0 = -2.0;
const CORNER_X1 = 2.0;
const CORNER_Y1 = 2.0;
const MAX_ITERATIONS = 512;
const NUMBER_OF_COLORS = 128;
const RGB_RED = 0;
const RGB_GREEN = 0;
const RGB_BLUE = 255;

// Wait for WASM to be initialized.
init().then(wasm => {

    // Create a new Mandelbrot object from Rust.
    // There are plenty of hardcoded values here. They should be given
    // by the user in the web-application.
    const mandel = Mandelbrot.new(CORNER_X0, CORNER_Y0, CORNER_X1, CORNER_Y1,
                                  canvas.width, canvas.height,
                                  MAX_ITERATIONS, NUMBER_OF_COLORS,
                                  RGB_RED, RGB_GREEN, RGB_BLUE);

    // Get access to the image that is handled from Rust.
    const data = new Uint8ClampedArray(wasm.memory.buffer, mandel.get_image(), canvas.width*canvas.height*4);

    // Create a new ImageData object that contains the image from Rust.
    // Since the memory is accessed by reference, we only need to create this once.
    const image = new ImageData(data, canvas.width, canvas.height)

    // Update image before rendering.
    mandel.update_image(0.0, 0.0, 1.0, 1.0);

    // Render the image in the canvas.
    ctx.putImageData(image, 0, 0);

    // Offer a slight abstraction, using a Point.
    // These are used to track the rectangle defined by the mouse actions.
    type Point = {x: number, y: number};
    let firstCorner: Point;
    let secondCorner: Point;

    // Define behaviour when mouse button is pressed.
    canvas.addEventListener('mousedown', e => {
        // Detect the first Corner of the new region.
        firstCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }
    });

    // Define behaviour when mouse button is released.
    canvas.addEventListener('mouseup', e => {
        // Exit if mouse button was not pressed.
        // This could happen if you press the mouse button outside
        // the canvas, and then hover over and release over the canvas.
        if (firstCorner === undefined) {
            return
        }

        // Detect the second corner of the new region.
        // There is no check that the corners are oriented.
        secondCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }

        // Update the image in Rust.
        mandel.update_image(firstCorner.x, firstCorner.y, secondCorner.x-firstCorner.x, secondCorner.y-firstCorner.y);

        // Render the updated image.
        ctx.putImageData(image, 0, 0);

        // Unset the corners.
        firstCorner = undefined;
        secondCorner = undefined;
    });

    // Define behaviour when mouse leaves the canvas while mouse button is pressed.
    canvas.addEventListener('mouseout', e => {
        // Unset the corners
        firstCorner = undefined;
        secondCorner = undefined;

        // Render the image to erase any rendered rectangle.
        ctx.putImageData(image, 0, 0);
    });

    // Define behaviour when mouse is moved.
    canvas.addEventListener('mousemove', e => {
        // Exit if mouse button was not pressed.
        if (firstCorner === undefined) {
            return
        }

        // Detect the mouse position.
        secondCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }

        // Render the image to remove any rectangles.
        ctx.putImageData(image, 0, 0);

        // Render a rectangle from the first and second corner.
        ctx.lineWidth = 5;
        ctx.strokeStyle = 'cyan';
        ctx.strokeRect(
            firstCorner.x*canvas.width,
            firstCorner.y*canvas.height,
            (secondCorner.x-firstCorner.x)*canvas.width,
            (secondCorner.y-firstCorner.y)*canvas.height)
    });
})
