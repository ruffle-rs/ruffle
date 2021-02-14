const fs = require("fs");
const path = require("path");
const { build, zip, createManifest } = require("./build_utils");

(async () => {
    console.log("Creating generic extension...");

    const dist = path.resolve(__dirname, "../dist");
    if (!fs.existsSync(dist)) {
        fs.mkdirSync(dist);
    }

    const { version } = require("../package.json");
    const manifest = createManifest({ version });

    await build();
    await zip(path.resolve(dist, "ruffle_extension.zip"), manifest);

    console.log("Generic extension zip has been built!");
})();
