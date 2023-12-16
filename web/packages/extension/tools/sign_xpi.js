import fsSync from "fs";
import fs from "fs/promises";
import url from "url";
import tempDir from "temp-dir";
import { signAddon } from "sign-addon";
import { Client as AMOClient } from "sign-addon/lib/amo-client.js";

/**
 * @param {string} apiKey
 * @param {string} apiSecret
 * @param {string} extensionId
 * @param {string} unsignedPath
 * @param {string} version
 * @param {string} destination
 * @param {string} sourcePath
 * @param {string} sourceTag
 */
async function sign(
    apiKey,
    apiSecret,
    extensionId,
    unsignedPath,
    version,
    destination,
    sourcePath,
    sourceTag,
) {
    const result = await signAddon({
        xpiPath: unsignedPath,
        version,
        apiKey,
        apiSecret,
        id: extensionId,
        downloadDir: tempDir,
    });

    // Since sign-addon doesn't support source upload, let's make the request
    // ourselves. We aren't actually using any API methods on AMOClient, just
    // the authentication mechanism, so this should be safe.
    const client = new AMOClient({
        apiKey,
        apiSecret,
        apiUrlPrefix: "https://addons.mozilla.org/api/v5",
        downloadDir: tempDir,
    });

    // NOTE: The extension ID is already wrapped in curly braces in GitHub.
    var sourceCodeUpload = client.patch({
        url: `/addons/addon/${encodeURIComponent(
            extensionId,
        )}/versions/${encodeURIComponent(version)}/`,
        formData: {
            source: fsSync.createReadStream(sourcePath),
        },
    });

    var notesUpload = client.patch({
        url: `/addons/addon/${encodeURIComponent(
            extensionId,
        )}/versions/${encodeURIComponent(version)}/`,
        json: {
            compatibility: {
                firefox: {
                    min: "84.0",
                },
                android: {
                    min: "120.0",
                },
            },
            approval_notes: `This version was derived from the source code available at https://github.com/ruffle-rs/ruffle/releases/tag/${sourceTag} - a ZIP file from this Git tag has been attached. If you download it yourself instead of using the ZIP file provided, make sure to grab the reproducible version of the ZIP, as it contains versioning information that will not be present on the main source download.\n\
\n\
We highly recommend using the Docker build workflow. You can invoke it using the following three commands:\n\
\n\
rm -rf web/docker/docker_builds/*\n\
docker build --tag ruffle-web-docker -f web/docker/Dockerfile .\n\
docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages\n\
\n\
These commands are run at the root of the project. The compiled XPI will be in web/docker/docker_builds/packages/extension/dist/firefox_unsigned.xpi. Please take care to use this file (and not the surrounding packages directory) when comparing against the extension submission.\n\
\n\
Alternatively, you may also attempt building using npm and cargo. However, this workflow is more complicated to set up. It is documented here:\n\
https://github.com/ruffle-rs/ruffle/blob/master/web/README.md\n\
\n\
Note that the commands for the npm/cargo workflow are run in the web subdirectory. If you're working with the Dockerfile you should be in the root of the project.\n\
\n\
The compiled version of this extension was built on Ubuntu 22.04 by our CI runner.\n\
\n\
As this is indeed a complicated build process, please let me know if there is anything I can do to assist.`,
        },
    });

    try {
        await Promise.all([sourceCodeUpload, notesUpload]);
        console.log("Successfully uploaded source code and approval notes");
    } catch (e) {
        console.error(`Got exception when uploading submission data: ${e}`);
    }

    if (!result.success && result.errorCode !== "ADDON_NOT_AUTO_SIGNED") {
        throw result;
    }

    if (result.downloadedFiles?.length === 1) {
        // Copy the downloaded file to the destination.
        // (Avoid `rename` because it fails if the destination is on a different drive.)
        const downloadedFile = /** @type {string} */ (
            result.downloadedFiles[0]
        );
        await fs.copyFile(downloadedFile, destination);
        await fs.unlink(downloadedFile);
    } else if (result.success) {
        console.warn(
            "Unexpected downloads for signed Firefox extension, expected 1.",
        );
        console.warn(result);
    }
}

try {
    if (
        process.env["MOZILLA_API_KEY"] &&
        process.env["MOZILLA_API_SECRET"] &&
        process.env["FIREFOX_EXTENSION_ID"] &&
        process.env["SOURCE_TAG"]
    ) {
        // TODO: Import as a JSON module once it becomes stable.
        const manifestPath = url.fileURLToPath(
            new URL("../assets/manifest.json", import.meta.url),
        );
        const manifest = JSON.parse(await fs.readFile(manifestPath, "utf8"));
        await sign(
            process.env["MOZILLA_API_KEY"],
            process.env["MOZILLA_API_SECRET"],
            process.env["FIREFOX_EXTENSION_ID"],
            /** @type {string} */ (process.argv[2]),
            manifest.version,
            /** @type {string} */ (process.argv[3]),
            /** @type {string} */ (process.argv[4]),
            process.env["SOURCE_TAG"],
        );
    } else {
        console.log(
            "Skipping signing of Firefox extension. To enable this, please provide MOZILLA_API_KEY, MOZILLA_API_SECRET, FIREFOX_EXTENSION_ID, and SOURCE_TAG environment variables",
        );
    }
} catch (error) {
    console.error("Error while signing Firefox extension:");
    console.error(error);
    process.exit(-1);
}
