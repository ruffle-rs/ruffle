const fs = require("fs");
const path = require("path");
const archiver = require("archiver");
const webpack = require("webpack");
const webpack_config = require("../webpack.config");

function build() {
    return new Promise((resolve, reject) => {
        const compiler = webpack(webpack_config());
        compiler.run((err) => {
            if (err) {
                return reject(err);
            }

            resolve();
        });
    });
}

function createManifest(overrides) {
    const manifest = require("../manifest.json");
    return { ...manifest, ...overrides };
}

async function zip(destination, manifest) {
    const output = fs.createWriteStream(destination);
    const archive = archiver("zip", {});

    output.on("close", function () {
        console.log(
            `Extension is ${archive.pointer()} total bytes when packaged`
        );
    });

    archive.on("error", function (err) {
        throw err;
    });

    archive.on("warning", function (err) {
        if (err.code === "ENOENT") {
            console.warn(`Warning whilst zipping extension: ${err}`);
        } else {
            throw err;
        }
    });

    archive.pipe(output);

    archive.directory(path.resolve(__dirname, `../assets`), "");
    archive.append(Buffer.from(JSON.stringify(manifest)), {
        name: "manifest.json",
    });

    await archive.finalize();
}

module.exports = {
    build,
    zip,
    createManifest,
};
