const fs = window.requires.fs;
export const readFileSync = fs.readFileSync;
export const writeFileSync = fs.writeFileSync;

const ipcRenderer = window.requires.ipcRenderer;

export function is_directory(path) {
    return ipcRenderer.invoke('is_directory', path)
        .then((ret) => {
            return ret;
        })
        .catch((err) => {
            alert(err);
        });
}
