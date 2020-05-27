const buildFirefox = require("./build_firefox");
const buildGeneric = require("./build_generic");

async function run() {
    await buildFirefox();
    await buildGeneric();

    console.log("All done! :)");
}

run().catch((error) => console.error(error));
