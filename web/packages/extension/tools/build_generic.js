const { build, zip, createManifest } = require("./utils");
const path = require("path");
const fs = require("fs");

async function run() {
    console.log("Creating generic extension...");

    const dist = path.resolve(__dirname, "../dist");
    fs.mkdirSync(dist);
    const version = require("../package.json").version;
    const manifest = createManifest({ version });

    await build();
    await zip(`${dist}/ruffle_extension.zip`, manifest);

    console.log("Generic extension zip has been built!");
}

module.exports = run;

if (!module.parent) {
    run().catch((error) => console.error(error));
}
