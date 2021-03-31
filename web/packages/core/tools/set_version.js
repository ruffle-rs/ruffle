const replace = require("replace-in-file");
const childProcess = require("child_process");

const version_number = process.env.npm_package_version;

const version_channel = process.env.CFG_RELEASE_CHANNEL || "nightly";

const build_date = new Date().toISOString();

let commitHash = "unknown";

try {
    commitHash = childProcess.execSync("git rev-parse HEAD").toString().trim();
} catch {
    console.log("Couldn't fetch latest git commit...");
}

const version_name =
    version_channel === "nightly"
        ? `nightly ${build_date.substr(0, 10)}`
        : process.env.npm_package_version;

const options = {
    files: "./pkg/**",
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
