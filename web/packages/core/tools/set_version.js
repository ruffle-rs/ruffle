const replace = require("replace-in-file");
const childProcess = require("child_process");
const fs = require("fs");

let version_number = process.env.npm_package_version;
let version_channel = process.env.CFG_RELEASE_CHANNEL || "nightly";
let build_date = new Date().toISOString();
const firefox_extension_id =
    process.env.FIREFOX_EXTENSION_ID || "ruffle@ruffle.rs";

let commitHash = "unknown";

try {
    commitHash = childProcess.execSync("git rev-parse HEAD").toString().trim();
} catch {
    console.log("Couldn't fetch latest git commit...");
}

let version_name =
    version_channel === "nightly"
        ? `nightly ${build_date.substr(0, 10)}`
        : process.env.npm_package_version;

let version_seal = {};

if (process.env.ENABLE_VERSION_SEAL === "true") {
    if (fs.existsSync("version_seal.json")) {
        // Using the version seal stored previously.
        version_seal = JSON.parse(fs.readFileSync("version_seal.json"));

        version_number = version_seal.version_number;
        version_name = version_seal.version_name;
        version_channel = version_seal.version_channel;
        build_date = version_seal.build_date;
        commitHash = version_seal.commitHash;
    } else {
        version_seal = {
            version_number: version_number,
            version_name: version_name,
            version_channel: version_channel,
            build_date: build_date,
            commitHash: commitHash,
            build_id: process.env.BUILD_ID,
            firefox_extension_id: firefox_extension_id,
        };

        fs.writeFileSync("version_seal.json", JSON.stringify(version_seal));
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
    to: [version_number, version_name, version_channel, build_date, commitHash],
};

replace.sync(options);
