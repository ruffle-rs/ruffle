const fs = require("fs");
const path = require("path");
const archiver = require("archiver");

async function zip(source, destination) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    const output = fs.createWriteStream(destination);
    const archive = archiver("zip", {});

    output.on("close", () => {
        console.log(
            `Extension is ${archive.pointer()} total bytes when packaged.`
        );
    });

    archive.on("error", (err) => {
        throw err;
    });

    archive.on("warning", (err) => {
        if (err.code === "ENOENT") {
            console.warn(`Warning whilst zipping extension: ${err}`);
        } else {
            throw err;
        }
    });

    archive.pipe(output);

    archive.directory(source, "");

    await archive.finalize();
}

(async () => {
    await zip(path.resolve(__dirname, "../assets"), process.argv[2]);
})();
