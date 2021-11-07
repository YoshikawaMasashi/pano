const {app, ipcMain, Menu, BrowserWindow, dialog } = require('electron');
const path = require('path');

let win;
const template = Menu.buildFromTemplate([
    {
      label: "ファイル",
      submenu: [
        {
          label:'import',
          submenu: [
            {
              label: 'Equirectangular',
              click:()=>{
                win.webContents.send("import_png", {});
              },
            },
          ]
        },
        {
          label:'export',
          submenu: [
            {
              label: 'Equirectangular',
              click:()=>{
                win.webContents.send("export_png", {});
              },
            },
          ]
        },
        { type:'separator' },
        { role:'close', label:'閉じる' },
      ]
    },
    {
      label: "編集",
      submenu: [
        { role:'undo',  label:'元に戻す' },
        { role:'redo',  label:'やり直す' },
        { type:'separator' },
        { role:'copy',  label:'コピー' },
        { role:'paste', label:'貼り付け' },
      ]
    }
]);
Menu.setApplicationMenu(template);

function createWindow() {
  win = new BrowserWindow({
    width: 1080,
    height: 1080,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
    }
  })

  global.setTimeout(() => {
    win.webContents.send("timer_tick", { message: "Hello World !" });
  }, 1000);

  win.loadFile(path.join(__dirname, 'index.html'))
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
