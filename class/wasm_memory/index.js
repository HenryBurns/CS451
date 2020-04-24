function main() {
    //alert("Hi");
    //memoryFromWasmToJs();
    memoryFromJsToWasm();
}

async function memoryFromWasmToJs() {
    // Download wasm
    const bytes = await fetch("target/wasm32-unknown-unknown/debug/wasm_memory.wasm");

    // Get byte array from bytes
    const wasmBytes = await bytes.arrayBuffer();

    // Instantiate a webassembly module
    const wasm = await WebAssembly.instantiate(wasmBytes, {});

    // We can now call exported functions from wasm

    wasm.instance.exports.memory_from_wasm_to_js();

    const wasmMemory =  new Uint8Array(
        wasm.instance.exports.memory.buffer, 0);

    alert("wasmMemory[0] = " + wasmMemory[0]);
}

async function memoryFromJsToWasm() {
    // We are going to directly modify the web
    // assembly memory from javascript.

    // Download wasm
    const bytes = await fetch("target/wasm32-unknown-unknown/debug/wasm_memory.wasm");

    // Get byte array from bytes
    const wasmBytes = await bytes.arrayBuffer();

    const wasmJsMemory = new WebAssembly.Memory(
        {
            initial: 10,
            maximum: 100
        }
    );

    // Instantiate a webassembly module
    const wasm = await WebAssembly.instantiate(
        wasmBytes,
        {js: {mem: wasmJsMemory}}
    );
    /**
    const wasm = await WebAssembly.instantiateStreaming(
        fetch("target/wasm32-unknown-unknown/debug/wasm_memory.wasm"),
        {js: {mem: wasmJsMemory}}
    );
    */

    // We want rust to add these numbers bc JS slow
    const to_add = new Set([1, 2, 3, 4, 5]);

    // Lets put this into memory directly.
    let jsArray = Uint8Array.from(to_add);

    const len = jsArray.length;

    // lets allocate some memory
    let wasmPointer = wasm.instance.exports.malloc(len);

    let wasmArray = new Uint8Array(
        wasm.instance.exports.memory.buffer,
        wasmPointer,
        len
    );
    wasmArray.set(jsArray);

    let sum = wasm.instance.exports.sum(wasmPointer, len);

    alert("wasm.sum(" + jsArray + ") = " + sum);
    // What now?
}
main();
