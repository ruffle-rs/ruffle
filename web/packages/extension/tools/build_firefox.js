const fs = require("fs");
const path = require("path");
const { default: signAddon } = require("sign-addon");
const { build, zip, createManifest } = require("./build_utils");

async function sign(
    api_key,
    api_secret,
    extension_id,
    unsigned_path,
    manifest,
    destination
) {
    const tempDir = require("temp-dir");
    const result = await signAddon({
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
                "Unexpected downloads for signed Firefox extension, expected 1."
            );
            console.warn(result);
        }
    }
}

(async () => {
    console.log("Creating Firefox extension...");

    const dist = path.resolve(__dirname, "../dist");
    if (!fs.existsSync(dist)) {
        fs.mkdirSync(dist);
    }

    const { version } = require("../package.json");
    const id =
        process.env.FIREFOX_EXTENSION_ID || "ruffle-player-extension@ruffle.rs";
    const manifest = createManifest({
        version,
        browser_specific_settings: { gecko: { id } },
    });

    await build();
    await zip(path.resolve(dist, "firefox_unsigned.xpi"), manifest);

    if (
        process.env.MOZILLA_API_KEY &&
        process.env.MOZILLA_API_SECRET &&
        process.env.FIREFOX_EXTENSION_ID
    ) {
        await sign(
            process.env.MOZILLA_API_KEY,
            process.env.MOZILLA_API_SECRET,
            process.env.FIREFOX_EXTENSION_ID,
            path.resolve(dist, "firefox_unsigned.xpi"),
            manifest,
            path.resolve(dist, "firefox.xpi")
        );
    } else {
        console.log(
            "Skipping signing of Firefox extension. To enable this, please provide MOZILLA_API_KEY, MOZILLA_API_SECRET and FIREFOX_EXTENSION_ID environment variables"
        );
    }

    console.log("Firefox extension has been built!");
})();
