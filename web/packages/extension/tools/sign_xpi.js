import fs from "fs/promises";
import { createRequire } from "module";
import tempDir from "temp-dir";
import { signAddon } from "sign-addon";
import { Client as AMOClient } from "sign-addon/lib/amo-client.js";

/**
 * Find a version of a given extension.
 *
 * The name is snarky because Mozilla's API really *should* hand us version IDs
 * automatically, but it doesn't. In A.M.O's defense we ARE mixing incompatible
 * API versions...
 *
 * Explicitly uses all_with_unlisted, which means this requires authentication
 * using a token that owns the extension in question.
 *
 * @param {AMOClient} client The client to use. Must have auth tokens.
 * @param {String} extensionId The extension ID to search. Must be wrapped in
 * curly braces.
 *
 * @param {String} version The version number to look for.
 *
 * @returns An A.M.O. version struct if found, otherwise null.
 */
async function find_the_version_we_just_submitted(
    client,
    extensionId,
    version
) {
    var page = 1;
    var result = {};

    while (page === 1 || result.count > 0) {
        result = await client.get({
            url: `/addons/addon/${encodeURIComponent(extensionId)}/versions`,
            qs: {
                filter: "all_with_unlisted",
                page: page,
            },
        });
        page += 1;

        for (var i = 0; i < result.results.length; i += 1) {
            if (result.results[i].version === version) {
                return result.results[i];
            }
        }
    }

    return null;
}

async function sign(
    apiKey,
    apiSecret,
    extensionId,
    unsignedPath,
    version,
    destination,
    sourcePath
) {
    const result = await signAddon({
        xpiPath: unsignedPath,
        version,
        apiKey,
        apiSecret,
        id: extensionId,
        downloadDir: tempDir,
    });

    //Since sign-addon doesn't support source upload, let's make the request
    //ourselves.
    const client = new AMOClient({
        apiKey,
        apiSecret,
        apiUrlPrefix: "https://addons.mozilla.org/api/v5",
        downloadDir: tempDir,
    });

    const submittedVersion = await find_the_version_we_just_submitted(
        client,
        extensionId,
        version
    );
    if (submittedVersion !== null) {
        console.debug(`Our version ID is ${submittedVersion.id}`);
        //NOTE: The extension ID is already wrapped in curly braces in GitHub
        var sourceCodeUpload = client.patch({
            url: `/addons/addon/${encodeURIComponent(
                extensionId
            )}/versions/${encodeURIComponent(submittedVersion.id)}/`,
            formData: {
                source: this._fs.createReadStream(sourcePath),
            },
        });

        const build_date = new Date().toISOString();

        var notesUpload = client.patch({
            url: `/addons/addon/${encodeURIComponent(
                extensionId
            )}/versions/${encodeURIComponent(submittedVersion.id)}/`,
            json: {
                approval_notes: `This version was derived from the source code available at https://github.com/ruffle-rs/ruffle/releases/tag/nightly-${build_date.substr(
                    0,
                    10
                )} - a ZIP file from this Git tag has been attached. If you download it yourself instead of using the ZIP file provided, make sure to grab the reproducible version of the ZIP, as it contains versioning information that will not be present on the main source download.\n\
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
            await Promise.all(sourceCodeUpload, notesUpload);
            console.log("Successfully uploaded source code and approval notes");
        } catch (e) {
            console.error(`Got exception when uploading submission data: ${e}`);
        }
    } else {
        console.error(`Version ${version} not found on addons.mozilla.org!`);
    }

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
            process.argv[3],
            process.argv[4]
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
