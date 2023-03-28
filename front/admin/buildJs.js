import * as esbuild from 'esbuild';
import * as fs from 'fs';



const ressourceFolder = async () => {
    // Define the paths of the source and destination folders
    const sourceFolder = 'ressource';
    const destFolder = 'dist/ressource';

    // Create the destination folder if it doesn't exist
    if (!fs.existsSync(destFolder)) {
        fs.mkdirSync(destFolder);
    }

    // Get the list of files and subfolders in the source folder
    const files = fs.readdirSync(sourceFolder);

    // Loop through each file/subfolder and copy it to the destination folder
    files.forEach(file => {
        const srcPath = path.join(sourceFolder, file);
        const destPath = path.join(destFolder, file);

        // Use fs.copyFileSync() to copy the file
        fs.copyFileSync(srcPath, destPath);

        // If the file is a subfolder, recursively copy it
        if (fs.lstatSync(srcPath).isDirectory()) {
            copyFolderRecursive(srcPath, destPath);
        }
    });

    // Recursively copy subfolders
    function copyFolderRecursive(srcFolder, destFolder) {
        // Create the destination folder if it doesn't exist
        if (!fs.existsSync(destFolder)) {
            fs.mkdirSync(destFolder);
        }

        // Get the list of files and subfolders in the source folder
        const files = fs.readdirSync(srcFolder);

        // Loop through each file/subfolder and copy it to the destination folder
        files.forEach(file => {
            const srcPath = path.join(srcFolder, file);
            const destPath = path.join(destFolder, file);

            // Use fs.copyFileSync() to copy the file
            fs.copyFileSync(srcPath, destPath);

            // If the file is a subfolder, recursively copy it
            if (fs.lstatSync(srcPath).isDirectory()) {
                copyFolderRecursive(srcPath, destPath);
            }
        });
    }
}


const build = async () => {

    //create the folder if not exist
    if (!fs.existsSync('dist')) {
        fs.mkdirSync('dist');
    }


    //copy the index.html to the dist folder
    fs.copyFileSync('index.html', 'dist/index.html');

    // copy ressource folder to the dist folder
    // await ressourceFolder();

    // Build the JS
    await esbuild.build({
        entryPoints: ['app/index.js'],
        bundle: true,
        minify: true,
        sourcemap: true,
        target: ['chrome58'],
        outfile: 'dist/index.js',
    });

    // Build the CSS
    await esbuild.build({
        entryPoints: ['style/style.css'],
        bundle: true,
        outfile: 'dist/style.css',
    });
}

build();


