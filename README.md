# WASM Steganography demonstrator

The tool is available online in its Github Page: [https://cristianmila.github.io/wasm-steganography-demo/](https://cristianmila.github.io/wasm-steganography-demo/)


It runs fully inside the client's browser, so neither images or secrets go outbounds. You can test it offline.

## ⚠️ Important Disclaimer

This project is an **educational demonstrator** created to explore and learn about WebAssembly (WASM). Its primary goal is to showcase how a single piece of Rust code, compiled to a WASM component, can be executed across different environments: a native CLI, a .NET Web API, and a browser-based Angular application.

**This is NOT a security tool.** The steganography technique used is a basic Least Significant Bit (LSB) implementation on 24-bit BMP files. It does not involve any encryption, and the hidden data is merely obfuscated. Do not use this for any sensitive information.

## About the project

This repository contains several projects that all leverage a central WASM component for steganography:

* **`/wasm`**: The core logic written in Rust. It exposes functions to encode and decode a string message into the pixels of a BMP image.
* **`/cli`**: A command-line tool written in Rust that uses the WASM component via the `wasmtime` crate.
* **`/webapi`**: A .NET 10 Web API that demonstrates how to execute the WASM module from a C# backend.
* **`/webtool`**: An Angular 20 webapp that runs the very same logic, transpiled from the WASM component, directly in the browser.

## Building from source

Check the [pipeline of the project](.github/workflows/pipeline.yml) to get detailed instructions.

These are some notable requirements:
- `wasm`: 
    - `wasm32-wasip1` target:
        ```bash
        rustup target add wasm32-wasip1
        ```
    - `wit-bindgen` cargo tool to generate the bindings from the `.wit` file:
        ```bash
        wit-bindgen rust --out-dir src/bindings.rs ./wit/world.wit
        ```
- `webtool`:
    - `jco` tool from `npm`. Read on for usage details or check the pipeline.
- `webapi`:
    - The environment variable `WASM_STEGANOGRAPHY_FILE_PATH` must be set to the `.wasm` **module**. It's checked at *runtime*.
- `cli`:
    - The environment variable `WASM_STEGANOGRAPHY_FILE_PATH` must be set to the `.wasm` **component**. It's checked at **compile time** in order to embed it as default WASM file and distribute the application as a dependency-free standalone executable. An optional `--wasm` parameter can be used to specify a different implementation of the `.wasm` **component**.

## Key learnings & technical deep dive

This project served as a learning journey into the current state of the WebAssembly ecosystem. Here are some of the key findings and challenges encountered.

### `std` vs. `no_std`

The initial goal was to create a `no_std` WASM module to ensure the smallest possible file size and maximum portability.

* **Initial size**: With `no_std`, the compiled `.wasm` file was a mere **33 KB**.
* **The challenge**: Finding a `no_std` compatible crate for manipulating BMP image pixels proved difficult. The `tinybmp` crate was evaluated but did not provide the necessary mutable access to pixel data.

The decision was made to switch back to using the feature-rich `image` crate. This crate depends on Rust's standard library (`std`), which in turn added WASI (WebAssembly System Interface) dependencies to the component. This increased the final file size to **~4.5 MB**, but provided the required functionality. Maintainers of `image` [are working on a `no_std` feature](https://github.com/image-rs/image/issues/1184), but it's not available as of now.

### WASM Modules vs. The Component Model

The project switched from compiling a vanilla WASM module to using the **WASM Component Model** via `cargo-component`. 

The Component Model simplifies the interface between the "guest" (our WASM) and the "host" (the runtime). By defining the public interface in a `.wit` file, we get strongly-typed, easy-to-use function signatures without manually handling memory pointers, which is a significant improvement over the more primitive module-level interactions. 

It is a much more convenient and familiar way to interact with WASM, as the WASM specification only defines numeric types. Things like passing a `string`, which are commonplace, involved handling a pointer and the size of the string. Same with arrays. For example, to be able to return an array, one should return a u64 integer with the following "encoding":
- Most significant 32 bits representing the pointer to the beggining of the array in the module's linear memory as a u32 integer.
- Least significant 32 bits representing the length of the array as a u32 integer.

You can see this technique applied in the [webapi](webapi/src/Webapi/SteganographyWasmModule.cs) file. Read on for further details.

#### In the Browser (Angular)

Browsers do not yet natively support the WASM Component Model.

Fortunately, the Bytecode Alliance's `jco` tool can be used to **transpile** the WASM component into a compatible WASM module and the necessary JavaScript "glue code". This "glue code" is source generated from the `.wasm` **component** -that is, built using `cargo-component`. It provides all the bindings required to work with the functions exported from the WASM component in an idiomatic way using complex types.

**Gotcha**: With the default parameters, the transpiled JS output uses Top-Level Await (TLA), which is not compatible with Angular's test runner `karma` as of v20. The following parameters were required to generate compatible bindings:
```bash
jco transpile --tla-compat --no-nodejs-compat <component>.wasm -o <output_dir>
```

#### On the Server (.NET)

The official `Wasmtime` NuGet package for .NET presented its own challenges. As of late 2025, the .NET runtime for `wasmtime` **does not support the WASM Component Model**. [There is an ongoing effort to add support](https://github.com/bytecodealliance/wasmtime-dotnet/issues/324), but it is not ready as of yet. Because of this, the manual approach was required. 

To get the functionality working in the C# Web API, the Rust code had to be compiled specifically as a **WASI v1 module**.
```bash
cargo build --target wasm32-wasip1 --release
```
This unfortunately breaks the "single artifact" goal of this project for now, as the .NET host requires a `.wasm` **module** file, not a component. than the Rust CLI or the transpiled web version.

#### Regarding `interface` in WIT definitions and Linux

WIT files allow the definition of interfaces declared as follows:
```wit
interface funcs {
	encode-secret-into-bmp: func(secret: string, image: list<u8>) -> list<u8>;
	decode-secret-from-bmp: func(image: list<u8>) -> string;
}

world steganography {
	export funcs;
}
```

This proved to work on Windows, but failed when compiling from Linux to execute the test collection of `cli`. This is the `version-script` generated by `rustc` targeting `x86_64-unknown-linux-gnu`:
```
{
  global:
    cabi_post_local:steganography/funcs#decode-secret-from-bmp;
    cabi_post_local:steganography/funcs#encode-secret-into-bmp;
    local:steganography/funcs#decode-secret-from-bmp;
    local:steganography/funcs#encode-secret-into-bmp;
    cabi_realloc_wit_bindgen_0_44_0;

  local:
    *;
};
```
The `ld` linker fails to link the functions because their names contain forbidden characters. Those '#' and '/' caracters are generated by `wit-bindgen` when using the `interface` export approach in the source `.wit` file. Flattening the `.wit` definition to export directly the functions solved the issue.

## Future Work

* **Format support**: Add support for other image formats, such as JPEG. Maybe add support for some audio format.
* **Improve error communication and handling**.
