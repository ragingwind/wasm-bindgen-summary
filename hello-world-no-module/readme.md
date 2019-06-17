# Hello World

> Built with no-module option, which does not support [local js snippets](https://rustwasm.github.io/docs/wasm-bindgen/reference/js-snippets.html) and generate ES module for web

```sh
# build
wasm-pack build --target no-modules

# run with http-server via npm
npx http-server .
```