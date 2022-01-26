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

### Webpack
In order to have a reactive development environment, I use ```webpack```.
One thing that drives me crazy is that the CSS-code is written directly in the HTML code. In order to use a separate CSS-file, we need to configure ```webpack``` to keep track of CSS-files. There are a few npm packages to install, and then it should work. Maybe not sufficiently important (here) to make the effort, but I still want to explore ```webpack``` a bit more.

### Extend to other iterative procedures
One could consider looking att different iterative schemes, like Julia-sets etc. The purpose of this project is mainly to get things working with Rust-wasm-typescript, with something that is computationally heavy.

### Better (less minimalistic) User Interface
The resulting webpage consist of a single canvas, that have the mandelbrot image, and where you can zoom in using the mouse. All parameters (colorscheme etc) are hardcoded in the typescript program. It should be easy to make some buttons, to make it a bit more user friendly.

### Concurrency improvement in Rust
The code is only using a single thread. Since the escape iteration detection is point-wise, this is a perfect example of an application that can be trivially parallelized. I should investigate how to do this in Rust.


# Installation Steps

1)  First step to initialize the project: ```cargo init --lib mandelbrot```.
2)  Create a directory ```mandelbrot/www```.

Move in to the ```mandelbrot/www``` directory.

3)  Initialize a node-project: ```npm init -y```.
4)  Install webpack ```npm install --save webpack webpack-cli copy-webpack-plugin```.
5)  Install development server: ```npm install --save-dev webpack-dev-server```.
6)  Create a ```.gitignore``` in ```www``` to avoid storing ```node_modules```.
7)  Create a sub-directory ```public```.
8)  Create ```index.html```, ```index.js``` and ```bootstrap.js``` in ```www```.
9)  Create and configure ```webpack.config.js```.
10) Add ```"dev": "webpack-dev-server"``` to the node configuration (```package.json```).
11) Add ```"mandelbrot": "file:../pkg",``` to ```package.json```.
In order to use typescript (from ~/www):

1) ```npm install --save typescript ts-loader```.
2) Configure ```tsconfig.json```.