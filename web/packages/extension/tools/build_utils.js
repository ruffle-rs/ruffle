const fs = require("fs");
const path = require("path");
const archiver = require("archiver");
const webpack = require("webpack");
const webpack_config = require("../webpack.config");

function build() {
    return new Promise((resolve, reject) => {
        webpack(webpack_config(), (err, stats) => {
            if (err) {
                console.error(err);
                reject();
                return;
            }
            if (stats.hasErrors()) {
                console.error(stats.toString({ colors: true }));
                reject();
                return;
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

    output.on("close", () => {
        console.log(
            `Extension is ${archive.pointer()} total bytes when packaged.`
        );
    });

    archive.on("error", (err) => {
        throw err;
    });

    archive.on("warning", (err) => {
        if (err.code !== "ENOENT") {
            throw err;
        }
        console.warn(`Warning whilst zipping extension: ${err}`);
    });

    archive.pipe(output);

    archive.directory(path.resolve(__dirname, "../assets"), "");
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
