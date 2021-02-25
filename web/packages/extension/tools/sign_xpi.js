const fs = require("fs");
const { signAddon } = require("sign-addon");

async function sign(
    apiKey,
    apiSecret,
    extensionId,
    unsignedPath,
    manifest,
    destination
) {
    const tempDir = require("temp-dir");
    const result = await signAddon({
        xpiPath: unsignedPath,
        version: manifest.version,
        apiKey,
        apiSecret,
        id: extensionId,
        downloadDir: tempDir,
    });

    if (!result.success) {
        throw result;
    }

    if (result.downloadedFiles.length === 1) {
        fs.renameSync(result.downloadedFiles[0], destination);
    } else {
        console.warn(
            "Unexpected downloads for signed Firefox extension, expected 1."
        );
        console.warn(result);
    }
}

(async () => {
    if (
        process.env.MOZILLA_API_KEY &&
        process.env.MOZILLA_API_SECRET &&
        process.env.FIREFOX_EXTENSION_ID
    ) {
        // TODO: read from unsigned xpi.
        const manifest = require("../assets/manifest");
        await sign(
            process.env.MOZILLA_API_KEY,
            process.env.MOZILLA_API_SECRET,
            process.env.FIREFOX_EXTENSION_ID,
            process.argv[2],
            manifest,
            process.argv[3]
        );
    } else {
        console.log(
            "Skipping signing of Firefox extension. To enable this, please provide MOZILLA_API_KEY, MOZILLA_API_SECRET and FIREFOX_EXTENSION_ID environment variables"
        );
    }
})();
