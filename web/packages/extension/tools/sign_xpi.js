import fs from "fs/promises";
import { createRequire } from "module";
import tempDir from "temp-dir";
import { signAddon } from "sign-addon";

async function sign(
    apiKey,
    apiSecret,
    extensionId,
    unsignedPath,
    version,
    destination
) {
    const result = await signAddon({
        xpiPath: unsignedPath,
        version,
        apiKey,
        apiSecret,
        id: extensionId,
        downloadDir: tempDir,
    });

    if (!result.success) {
        throw result;
    }

    if (result.downloadedFiles.length === 1) {
        // Copy the downloaded file to the destination.
        // (Avoid `rename` because it fails if the destination is on a different drive.)
        await fs.copyFile(result.downloadedFiles[0], destination);
        await fs.unlink(result.downloadedFiles[0]);
    } else {
        console.warn(
            "Unexpected downloads for signed Firefox extension, expected 1."
        );
        console.warn(result);
    }
}

try {
    if (
        process.env.MOZILLA_API_KEY &&
        process.env.MOZILLA_API_SECRET &&
        process.env.FIREFOX_EXTENSION_ID
    ) {
        // TODO: Import as a JSON module once it becomes stable.
        const require = createRequire(import.meta.url);
        const { version } = require("../assets/manifest.json");
        await sign(
            process.env.MOZILLA_API_KEY,
            process.env.MOZILLA_API_SECRET,
            process.env.FIREFOX_EXTENSION_ID,
            process.argv[2],
            version,
            process.argv[3]
        );
    } else {
        console.log(
            "Skipping signing of Firefox extension. To enable this, please provide MOZILLA_API_KEY, MOZILLA_API_SECRET and FIREFOX_EXTENSION_ID environment variables"
        );
    }
} catch (error) {
    console.error("Error while signing Firefox extension:");
    console.error(error);
    process.exit(-1);
}
