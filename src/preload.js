const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld(
  "requires", {
    fs : require("fs"),
    ipcRenderer : ipcRenderer,
  }
);

contextBridge.exposeInMainWorld(
  'api', {
    on: (channel, callback) => ipcRenderer.on(channel, (event, argv)=>callback(event, argv))
  }
)
