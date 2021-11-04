import init, {greet} from "../pano-rs/pkg/pano.js";
init()
  .then(() => {
     greet("WebAssembly")
  });
