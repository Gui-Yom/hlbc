#!/usr/bin/env sh
cd ..
cargo build --profile release-web --target wasm32-unknown-unknown
cd web
wasm-bindgen ../../target/wasm32-unknown-unknown/release-web/hlbc-gui.wasm --out-dir dist --target web --no-typescript
wasm-opt dist/hlbc-gui_bg.wasm -o dist/output.wasm -Os
rm dist/hlbc-gui_bg.wasm
mv dist/output.wasm dist/hlbc-gui_bg.wasm
cp index.html dist/index.html
