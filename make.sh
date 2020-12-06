set -euxo pipefail

cargo build --target=wasm32-unknown-unknown
rm -rf target/generated
wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/feo-boy.wasm
python3 -m http.server
