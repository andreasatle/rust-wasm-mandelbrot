# Mandelbrot set

We consider the sequence

$z_{n+1} = z_n^2 + c$, with $z_0=0$,

where $c$ is a complex number.

The Mandelbrot set are all $c$ such that the sequence is bounded.

The Mandelbrot sequence diverges if $|z_n|\ge 2$ and $|z_n|\ge |c|$ for any $n$.

The reverse triangle inequality, $|x+y| \ge |x| - |y|$.

$|z_{n+1}|=|z_n^2+c|\ge |z_n|^2-|c| \ge 2|z_n|-|z_n| = |z_n|$.

We have shown that such sequences are unbounded and do not belong complement of the Mandelbrot set.

A corollary is that the Mandelbrot set is contained in the circle $|c| \le 2$.

![Example Image](/img/Mandelbrot-Image.png)
*Snapshot of an early zoom in the Mandelbrot image.*

## Implementation details
The escape iteration is the first iteration where $|z|\ge 2$.
We compute the escape iteration for each point in the image.
The final task is then to decide how to map the escape iteration to a color. A major difficulty with any Mandelbrot computation is to balance the colors properly. We compute the frequency (occurances) of each escape iteration, and compute the cumulative sum of the frequencies. After we bin the different escape frequencies to have an approximately linear spread.

On the typescript side, the image is stored as an ```ImageData```, consisting of a ```UInt8ClampedArray```. The ```RGBA``` values are stored in steps of 4, repeating like
```
R,G,B,A, R,G,B,A, ...
```

On the Rust-side, I naively used a vector with u32, and used bit manipulations to get the bits for each color etc. This turned out to be a nightmare, due to little endian-problem. I switched to using a vec of u8, and then it worked.

## Things to improve
In order to have a reactive development environment, I use ```webpack```.
One thing that drives me crazy is that the CSS-code is written directly in the HTML code. In order to use a separate CSS-file, we need to configure ```webpack``` to keep track of CSS-files. There are a few npm packages to install, and then it should work. Maybe not sufficiently important (here) to make the effort, but I still want to explore ```webpack``` a bit more.

One could consider looking att different iterative schemes, like Julia-sets etc. The purpose of this project is mainly to get things working with Rust-wasm-typescript, with something that is computationally heavy.

The resulting webpage consist of a single canvas, that have the mandelbrot image, and where you can zoom in using the mouse. All parameters (colorscheme etc) are hardcoded in the typescript program. It should be easy to make some buttons, to make it a bit more user friendly.

