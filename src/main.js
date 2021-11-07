const {app, ipcMain, BrowserWindow, dialog } = require('electron');
const path = require('path');

function createWindow() {
  const win = new BrowserWindow({
    width: 1080,
    height: 1080,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
    }
  })

  win.loadFile('index.html');
  win.webContents.openDevTools();
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') app.quit()
})

ipcMain.handle('showOpenDirectoryDialog', async (event) => {
  let filename = dialog.showOpenDialogSync(null, {
      properties: ['openDirectory', 'createDirectory'],
      title: 'Select a directory',
      defaultPath: '.'
  });
  return filename;
});

ipcMain.handle('showOpenPngDialog', async (event) => {
  let filename = dialog.showOpenDialogSync(null, {
      properties: ['openFile'],
      title: 'Select a png image',
      defaultPath: '.',
      filters: [
          {name: 'png file', extensions: ['png']}
      ]
  });
  return filename;
});

ipcMain.handle('showSavePngDialog', async (event) => {
  let filename = dialog.showSaveDialogSync(null, {
      properties: ['showOverwriteConfirmation'],
      title: 'Select a png image',
      defaultPath: '.',
      filters: [
          {name: 'png file', extensions: ['png']}
      ]
  });
  return filename;
});
