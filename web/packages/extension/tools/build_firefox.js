const { build, zip } = require("./utils");
const path = require("path");
const signAddon = require("sign-addon").default;
const fs = require("fs");

async function sign(
    api_key,
    api_secret,
    extension_id,
    unsigned_path,
    manifest,
    destination
) {
    const tempDir = require("temp-dir");
    let result = await signAddon({
        xpiPath: unsigned_path,
        version: manifest.version,
        apiKey: api_key,
        apiSecret: api_secret,
        id: extension_id,
        downloadDir: tempDir,
    });

    if (result.success) {
        if (result.downloadedFiles.length === 1) {
            fs.renameSync(result.downloadedFiles[0], destination);
        } else {
            console.warn(
                "Unexpected downloads for signed firefox extension, expected 1."
            );
            console.warn(result);
        }
    }
}

function createManifest(overrides) {
    const manifest = require("../manifest.json");
    return { ...manifest, ...overrides };
}

async function run() {
    console.log("Creating firefox extension...");

    const dist = path.resolve(__dirname, "../dist");
    fs.mkdirSync(dist);
    const version = require("../package.json").version;
    const manifest = createManifest({ version });

    await build();
    await zip(`${dist}/firefox_unsigned.xpi`, manifest);

    if (
        process.env.MOZILLA_API_KEY &&
        process.env.MOZILLA_API_SECRET &&
        process.env.FIREFOX_EXTENSION_ID
    ) {
        await sign(
            process.env.MOZILLA_API_KEY,
            process.env.MOZILLA_API_SECRET,
            process.env.FIREFOX_EXTENSION_ID,
            `${dist}/firefox_unsigned.xpi`,
            manifest,
            `${dist}/firefox.xpi`
        );
    } else {
        console.log(
            "Skipping signing of firefox extension. To enable this, please provide MOZILLA_API_KEY, MOZILLA_API_SECRET and FIREFOX_EXTENSION_ID environment variables"
        );
    }

    console.log("Firefox extension has been built!");
}

module.exports = run;

if (!module.parent) {
    run().catch((error) => console.error(error));
}
