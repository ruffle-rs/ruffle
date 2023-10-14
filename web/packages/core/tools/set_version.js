const replace = require("replace-in-file");
const childProcess = require("child_process");
const fs = require("fs");

let buildDate = new Date().toISOString();
let versionNumber = process.env.npm_package_version;
let versionChannel = process.env.CFG_RELEASE_CHANNEL || "nightly";
const firefoxExtensionId =
    process.env.FIREFOX_EXTENSION_ID || "ruffle@ruffle.rs";

let commitHash = "unknown";

try {
    commitHash = childProcess.execSync("git rev-parse HEAD").toString().trim();
} catch {
    console.log("Couldn't fetch latest git commit...");
}

let versionName =
    versionChannel === "nightly"
        ? `nightly ${buildDate.substr(0, 10)}`
        : process.env.npm_package_version;

let versionSeal = {};

if (process.env.ENABLE_VERSION_SEAL === "true") {
    if (fs.existsSync("version_seal.json")) {
        // Using the version seal stored previously.
        versionSeal = JSON.parse(fs.readFileSync("version_seal.json"));

        versionNumber = versionSeal.version_number;
        versionName = versionSeal.version_name;
        versionChannel = versionSeal.version_channel;
        buildDate = versionSeal.build_date;
        commitHash = versionSeal.commitHash;
    } else {
        versionSeal = {
            version_number: versionNumber,
            version_name: versionName,
            version_channel: versionChannel,
            build_date: buildDate,
            commitHash: commitHash,
            build_id: process.env.BUILD_ID,
            firefox_extension_id: firefoxExtensionId,
        };

        fs.writeFileSync("version_seal.json", JSON.stringify(versionSeal));
    }
}

const options = {
    files: "dist/**",
    from: [
        /%VERSION_NUMBER%/g,
        /%VERSION_NAME%/g,
        /%VERSION_CHANNEL%/g,
        /%BUILD_DATE%/g,
        /%COMMIT_HASH%/g,
    ],
    to: [versionNumber, versionName, versionChannel, buildDate, commitHash],
};

replace.sync(options);
