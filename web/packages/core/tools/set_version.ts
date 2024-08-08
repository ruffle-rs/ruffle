import { replaceInFileSync } from "replace-in-file";
import childProcess from "child_process";
import fs from "fs";
import path from "path";

let buildDate = new Date().toISOString();
let versionNumber = process.env["npm_package_version"] ?? "";
let versionChannel = process.env["CFG_RELEASE_CHANNEL"] || "none";
const firefoxExtensionId =
    process.env["FIREFOX_EXTENSION_ID"] || "ruffle@ruffle.rs";

let commitHash = "unknown";

try {
    commitHash = childProcess.execSync("git rev-parse HEAD").toString().trim();
} catch {
    console.log("Couldn't fetch latest git commit...");
}

let versionName =
    versionChannel === "nightly"
        ? `nightly ${buildDate.substr(0, 10)}`
        : versionNumber;

interface VersionInformation {
    version_number: string;
    version_name: string;
    version_channel: string;
    build_date: string;
    commitHash: string;
    version4: string;
    firefox_extension_id: string;
}

let versionSeal: VersionInformation;

if (process.env["ENABLE_VERSION_SEAL"] === "true") {
    const sealFile = path.resolve(__dirname, "../../../version_seal.json");
    if (fs.existsSync(sealFile)) {
        console.log("Using version seal");
        // Using the version seal stored previously.
        versionSeal = JSON.parse(
            fs.readFileSync(sealFile, { encoding: "utf8" }),
        ) as VersionInformation;

        versionNumber = versionSeal.version_number;
        versionName = versionSeal.version_name;
        versionChannel = versionSeal.version_channel;
        buildDate = versionSeal.build_date;
        commitHash = versionSeal.commitHash;
    } else {
        console.log("Creating version seal");
        versionSeal = {
            version_number: versionNumber,
            version_name: versionName,
            version_channel: versionChannel,
            build_date: buildDate,
            commitHash: commitHash,
            version4: process.env["VERSION4"] ?? "",
            firefox_extension_id: firefoxExtensionId,
        };

        fs.writeFileSync(sealFile, JSON.stringify(versionSeal));
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

replaceInFileSync(options);
