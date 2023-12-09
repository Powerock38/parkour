#!/usr/bin/env bash

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir . --target web ./target/wasm32-unknown-unknown/release/parkour.wasm

git stash --include-untracked
git checkout web
git checkout main -- assets/
rm parkour_bg.wasm parkour_bg.wasm.d.ts parkour.d.ts parkour.js
git stash pop

git add .
git commit -m "build"
git push

git checkout main
