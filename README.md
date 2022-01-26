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
