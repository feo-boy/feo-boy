#!/bin/sh

set -eux

cargo build --target=wasm32-unknown-unknown
rm -rf web/dist/feo-boy
wasm-bindgen --out-dir web/dist/feo-boy --target no-modules target/wasm32-unknown-unknown/debug/feo-boy.wasm
cd web
npm run build
cd dist
python3 -m http.server
