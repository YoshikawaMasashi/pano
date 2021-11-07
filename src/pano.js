import init from "./pkg/pano.js";
import * as wasm from './pkg/pano.js';
import { on_click_export_png } from './electron_on.js';

// https://github.com/anderejd/electron-wasm-rust-example
async function run() {
	await init('./pkg/pano_bg.wasm');
    wasm.start();

    window.api.on("export_png", (event, arg) => {
        on_click_export_png();
    });
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
