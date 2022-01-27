// typescript utility functions
// that can be used in rust-wasm.

// Wrapper routine to write on the console in the browser.
export function output_js(str: string) {
    console.log(str);
}
