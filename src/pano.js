import init from "../pano-rs/pkg/pano.js";
import * as wasm from '../pano-rs/pkg/pano.js';

// https://github.com/anderejd/electron-wasm-rust-example
async function run() {
	await init('../pano-rs/pkg/pano_bg.wasm');
    console.log(wasm);
}
run();
