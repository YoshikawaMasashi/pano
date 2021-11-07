const ipcRenderer = window.requires.ipcRenderer;

export function showOpenDirectoryDialog() {
    return ipcRenderer.invoke('showOpenDirectoryDialog')
        .then((data) => {
            if (data !== undefined) {
                console.log(data[0], data[0].length);
                return data[0];
            }
            return data;
        })
        .catch((err) => {
            alert(err);
        });
}

export function showOpenPngDialog() {
    return ipcRenderer.invoke('showOpenPngDialog')
        .then((data) => {
            if (data !== undefined) {
                console.log(data[0], data[0].length);
                return data[0];
            }
            return data;
        })
        .catch((err) => {
            alert(err);
        });
}

export function showSavePngDialog() {
    return ipcRenderer.invoke('showSavePngDialog')
        .then((data) => {
            return data;
        })
        .catch((err) => {
            alert(err);
        });
}
