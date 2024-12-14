import fs from "node:fs";
import crypto from "node:crypto";
import { setTimeout } from "node:timers/promises";
import axios from "axios";
import jwt from "jsonwebtoken";
import FormData from "form-data";

// This script implements the same basic procedure as described here:
// https://blog.mozilla.org/addons/2022/03/17/new-api-for-submitting-and-updating-add-ons/

function getJwtToken(apiKey: string, apiSecret: string) {
    const payload = {
        iss: apiKey,
        jti: crypto.randomBytes(32).toString("hex"),
        iat: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + 60,
    };

    return jwt.sign(payload, apiSecret, { algorithm: "HS256" });
}

async function submit(
    apiKey: string,
    apiSecret: string,
    extensionId: string,
    unsignedPath: string,
    sourcePath: string,
    sourceTag: string,
) {
    // The uploading, waiting for validation, and submitting parts could be done by this:
    // https://extensionworkshop.com/documentation/develop/getting-started-with-web-ext/
    // But since we're already poking directly at the API, might as well do those too...

    // For API docs, see: https://mozilla.github.io/addons-server/topics/api/addons.html
    const client = axios.create({
        baseURL: "https://addons.mozilla.org/api/v5/addons/",
    });

    console.log("Checking the status of the last submitted add-on version...");
    const versionsResponse = await client.get(
        `addon/${extensionId}/versions/`,
        {
            headers: {
                Authorization: `JWT ${getJwtToken(apiKey, apiSecret)}`,
            },
            params: {
                filter: "all_with_unlisted",
            },
        },
    );

    const lastVersion = versionsResponse.data.results[0];
    switch (lastVersion.file.status) {
        case "public":
            console.log("Looks like we're good to go!");
            break;
        case "unreviewed":
            console.log(
                "Last version still awaiting review, skipping submission.",
            );
            return;
        case "disabled":
            throw new Error(
                "Last version was either rejected, disabled, or not reviewed - skipping submission.",
            );
        default:
            throw new Error(
                "Last version has an unknown status: " + lastVersion.status,
            );
    }

    console.log("Uploading unsigned add-on...");
    const addonFormData = new FormData();
    addonFormData.append("channel", "listed");
    addonFormData.append("upload", fs.createReadStream(unsignedPath));

    const addonUploadResponse = await client.postForm(
        "upload/",
        addonFormData,
        {
            headers: {
                ...addonFormData.getHeaders(),
                Authorization: `JWT ${getJwtToken(apiKey, apiSecret)}`,
            },
        },
    );

    const uploadUuid = addonUploadResponse.data.uuid;
    console.log("Upload UUID: " + uploadUuid);

    console.log("Waiting for the upload to be processed...");
    // "The recommended polling interval is 5-10 seconds, making
    // sure your code times out after a maximum of 10 minutes."
    for (let i = 0; i < 42; i++) {
        console.log("Sleeping for a couple seconds...");
        await setTimeout(10000);

        const uploadDetailResponse = await client.get(`upload/${uploadUuid}/`, {
            headers: {
                Authorization: `JWT ${getJwtToken(apiKey, apiSecret)}`,
            },
        });

        if (!uploadDetailResponse.data.processed) {
            console.log("Not processed yet.");
            continue;
        }

        console.log("Processed! Validation messages:");
        console.log(uploadDetailResponse.data.validation.messages);

        if (uploadDetailResponse.data.valid) {
            break;
        } else {
            throw new Error("Validation failed");
        }
    }

    console.log("Creating a new version of the add-on...");
    const createResponse = await client.post(
        `addon/${extensionId}/versions/`,
        {
            upload: uploadUuid,
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
rm -rf web/docker/docker_builds/packages/*\n\
# Normally these commands:\n\
docker build --tag ruffle-web-docker -f web/docker/Dockerfile .\n\
docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages\n\
# OR alternatively, if you have to use 'sudo docker', make sure to use $(sudo docker ...) for the second docker command:\n\
sudo docker build --tag ruffle-web-docker -f web/docker/Dockerfile .\n\
sudo docker cp $(sudo docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages\n\
\n\
These commands are run at the root of the project. The compiled XPI will be in web/docker/docker_builds/packages/extension/dist/firefox_unsigned.xpi. Please take care to use this file (and not the surrounding packages directory) when comparing against the extension submission.\n\
\n\
Note that the wasm files may not match, but we have been informed previously by Mozilla that this is allowed. The JavaScript and all other files should match.
\n\
The compiled version of this extension was built on Ubuntu 22.04 by our CI runner.\n\
\n\
As this is indeed a complicated build process, please let me know if there is anything I can do to assist.`,
        },
        {
            headers: {
                "Content-Type": "application/json",
                Authorization: `JWT ${getJwtToken(apiKey, apiSecret)}`,
            },
        },
    );

    const version = createResponse.data.version;
    console.log("Created version: " + version);

    console.log("Uploading source code...");
    const sourceFormData = new FormData();
    sourceFormData.append("source", fs.createReadStream(sourcePath));

    const sourceUploadResponse = await client.patch(
        `addon/${extensionId}/versions/${version}/`,
        sourceFormData,
        {
            headers: {
                ...sourceFormData.getHeaders(),
                Authorization: `JWT ${getJwtToken(apiKey, apiSecret)}`,
            },
        },
    );
    console.log("Source file ID: " + sourceUploadResponse.data.id);

    console.log(
        "Add-on uploaded, a new version created, and source code uploaded successfully",
    );
}

async function main() {
    try {
        if (
            process.env["MOZILLA_API_KEY"] &&
            process.env["MOZILLA_API_SECRET"] &&
            process.env["FIREFOX_EXTENSION_ID"] &&
            process.env["SOURCE_TAG"]
        ) {
            await submit(
                process.env["MOZILLA_API_KEY"], // "user:12345678:123"
                process.env["MOZILLA_API_SECRET"], // 64 hexadecimal characters
                process.env["FIREFOX_EXTENSION_ID"], // "{UUID}"
                process.argv[2] ?? "", // "firefox_unsigned.xpi"
                process.argv[3] ?? "", // "reproducible-source.zip"
                process.env["SOURCE_TAG"], // "nightly-YYYY-MM-DD"
            );
        } else {
            console.log(
                "Skipping submission of Firefox extension. To enable this, please set the MOZILLA_API_KEY, MOZILLA_API_SECRET, FIREFOX_EXTENSION_ID, and SOURCE_TAG environment variables.",
            );
        }
    } catch (error) {
        console.error("Error while submitting Firefox extension:");
        console.error(error);
        process.exit(-1);
    }
}
main();
