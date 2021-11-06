import init from "../pano-rs/pkg/pano.js";
import * as wasm from '../pano-rs/pkg/pano.js';

// https://github.com/anderejd/electron-wasm-rust-example
async function run() {
	await init('../pano-rs/pkg/pano_bg.wasm');
    // wasm.show_panorama();
    wasm.start();
    /*
    var xrApp = new wasm.XrApp();
    xrApp.init()
        .then(res => {
            if (res) {
                console.log('init ok');
                xrApp.start();
            }
            else {
                console.log('init failed');
            }
        });
    */
}
run();
