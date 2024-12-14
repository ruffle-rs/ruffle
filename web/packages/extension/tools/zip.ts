import fs from "fs/promises";
import path from "path";
import url from "url";
import archiver from "archiver";

async function zip(source: string, destination: string) {
    await fs.mkdir(path.dirname(destination), { recursive: true });
    const output = (await fs.open(destination, "w")).createWriteStream();
    const archive = archiver("zip");

    output.on("close", () => {
        console.log(
            `Extension is ${archive.pointer()} total bytes when packaged.`,
        );
    });

    archive.on("error", (error) => {
        throw error;
    });

    archive.on("warning", (error) => {
        if (error.code === "ENOENT") {
            console.warn(`Warning whilst zipping extension: ${error}`);
        } else {
            throw error;
        }
    });

    archive.pipe(output);

    archive.directory(source, "");

    await archive.finalize();
}
const assets = url.fileURLToPath(new URL("../assets/", import.meta.url));
zip(assets, process.argv[2] ?? "").catch(console.error);
