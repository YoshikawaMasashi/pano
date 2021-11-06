import ReactDOM from 'react-dom';
import React, { Component } from 'react';

import init from "../../pano-rs/pkg/pano.js";
import * as wasm from '../../pano-rs/pkg/pano.js';

class App extends Component {
    render() {
        return (
        <div className="App">
            <canvas id="canvas" height="500" width="500"></canvas>
            <header className="App-header">
            <p>
                Edit <code>src/App.js</code> and save to reload.
            </p>
            <a
                className="App-link"
                href="https://reactjs.org"
                target="_blank"
                rel="noopener noreferrer"
            >
                Learn React
            </a>
            </header>
        </div>
        );
    }
}

// https://github.com/anderejd/electron-wasm-rust-example
async function run() {
    ReactDOM.render(<App />, document.getElementById('root'));

	await init('../pano-rs/pkg/pano_bg.wasm');
    console.log(wasm);
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
