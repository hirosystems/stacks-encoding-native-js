# stacks-encoding-native-js

_This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon)._


NodeJS library providing platform-native functions for decoding binary/wire formats from the Stacks blockchain. This includes Clarity values, transactions, post-conditions, Stacks and Bitcoin addresses, and more. All of which use ASM/SIMD where available.

### Supported platforms

* Linux (glibc) x86_64
* Linux (glibc) aarch64
* Linux (musl) x86_64
* Linux (musl) aarch64
* MacOS x86_64
* MacOS aarch64
* Windows x86_64

## Project Layout

The directory structure of this project is:

<pre>
/
├── Cargo.toml    # The Cargo <a href="https://doc.rust-lang.org/cargo/reference/manifest.html">manifest file</a>
├── package.json  # The npm <a href="https://docs.npmjs.com/cli/v7/configuring-npm/package-json">manifest file</a>
├── native/       # The <a href="https://nodejs.org/api/addons.html">Node addon</a> modules built by this project, these are <a href="https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries">dynamically-linked shared objects</a>
├── src/**/*.rs   # Directory containing the Rust source code for the project
|── index.ts      # Typescript definitions for the js interface exposed by the Node addon
|── loader.js     # Script to determine which addon file to load based on the executing target platform
|── loader.d.ts   # Type defintions for the functions exported by the Node addon
|── builder.js    # Script to build the native Node addon for the executing host platform
├── tests/*.ts    # Js/ts unit tests, primarily testing the Neon (rust<->nodejs) interop layer
└── perf-tests/   # Scripts to run performance benchmarks used by commands in package.json
</pre>

The Rust source code inside the `src/**/deserialize.rs` files are responsible for deserializing the Stacks blockchain wire/binary formats defined in [SIP-005](https://github.com/stacksgov/sips/blob/main/sips/sip-005/sip-005-blocks-and-transactions.md). 

## NPM Library Bundling

The Node addon modules for all supported platforms are compiled by [CI](.github/workflows/build.yml) and bundled inside the npm package. The native binary files are small enough that the bundled npm package is an acceptable ~20 MB in size. 