const fs = window.requires.fs;
export const readFileSync = fs.readFileSync;
export const writeFileSync = fs.writeFileSync;
export function is_directory(path) {
    fs.stat("C:/Users/earne/Desktop/pano_project/pano/src", function  (err, stats) {
        if (err) throw err;
        console.log(stats);
        if (stats.isDirectory()) {
            console.log('This is a directory');
        }
    });
    
    fs.stat(path, function (err, stats) {
        if (err) throw err;
        if (stats.isDirectory()) {
            console.log('This is a directory');
        }
    });
    return false;
}
