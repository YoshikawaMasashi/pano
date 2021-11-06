const { contextBridge } = require("electron");

contextBridge.exposeInMainWorld(
  "requires", {
    fs : require("fs"),
  }
);