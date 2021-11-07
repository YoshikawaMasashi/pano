import init from "./pkg/pano.js";
import * as wasm from './pkg/pano.js';

// https://github.com/anderejd/electron-wasm-rust-example
async function run() {
	await init('./pkg/pano_bg.wasm');
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
