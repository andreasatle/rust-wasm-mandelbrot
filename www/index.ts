// Import rust code.
import init, { Mandelbrot} from "mandelbrot";

// Setup the canvas from HTML.
const canvas = document.getElementById("mandelbrot-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
canvas.height = 2048;
canvas.width = 2048;

init().then(wasm => {

    // Hard-Coded rectangle TODO Fix
    const mandel = Mandelbrot.new(-2.0, -2.0, 2.0, 2.0, canvas.width, canvas.height, 512, 128, 255, 0, 0);
    const data = new Uint8ClampedArray(wasm.memory.buffer, mandel.get_image(), canvas.width*canvas.height*4);
    const image = new ImageData(data, canvas.width, canvas.height)

    mandel.update_image(0.0, 0.0, 1.0, 1.0);
    ctx.putImageData(image, 0, 0);

    type Point = {x: number, y: number};
    let firstCorner: Point;
    let secondCorner: Point;
    canvas.addEventListener('mousedown', e => {
        firstCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }
    });
    
    canvas.addEventListener('mouseup', e => {
        if (firstCorner === undefined) {
            return
        }
        secondCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }

        mandel.update_image(firstCorner.x, firstCorner.y, secondCorner.x-firstCorner.x, secondCorner.y-firstCorner.y);
        ctx.putImageData(image, 0, 0);

        firstCorner = undefined;
        secondCorner = undefined;
    });
    canvas.addEventListener('mouseout', e => {
        firstCorner = undefined;
        secondCorner = undefined;

        ctx.putImageData(image, 0, 0);
    });
    canvas.addEventListener('mousemove', e => {
        if (firstCorner === undefined) {
            return
        }
        secondCorner = {
            x: (e.clientX-canvas.offsetLeft)/canvas.offsetWidth,
            y: (e.clientY-canvas.offsetTop)/canvas.offsetHeight
        }
        ctx.putImageData(image, 0, 0);

        ctx.lineWidth = 5;
        ctx.strokeStyle = 'cyan';
        ctx.strokeRect(
            firstCorner.x*canvas.width,
            firstCorner.y*canvas.height,
            (secondCorner.x-firstCorner.x)*canvas.width,
            (secondCorner.y-firstCorner.y)*canvas.height)
    });
})
