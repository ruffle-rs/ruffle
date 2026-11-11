import { encode } from "base85";
import { minify as minifyHtml } from "html-minifier-terser";
import { minify_sync as minifyTerser } from "terser";
import { join } from "node:path";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { JSDOM } from "jsdom";

const suffixToMime = {
    ".wasm": "application/wasm",
    ".js": "application/javascript",
    ".png": "image/png",
    ".svg": "image/svg+xml",
    ".woff2": "font/woff2",
};

// encode base85-gzip files
function generateDynSrcContent(distDir, resRegex) {
    const resList = readdirSync(distDir);
    let generated = {};
    for (const resName of resList) {
        const suffix = resName.substring(resName.lastIndexOf("."));
        if (resRegex.test(resName)) {
            let fileContent = readFileSync(join(distDir, resName));
            fileContent = gzipSync(fileContent, { level: 9 });
            const fileLength = fileContent.length;
            // the file size for z85 needs to be a multiple of 4
            if (fileLength % 4 != 0) {
                let newFileContent = Buffer.alloc(Math.ceil(fileLength / 4) * 4);
                fileContent.copy(newFileContent);
                fileContent = newFileContent;
            }
            const encoded = encode(fileContent, "z85");
            if (!encoded) throw new Error("Failed to encode", resName);
            generated[resName] = [fileLength, encoded, suffixToMime[suffix] || "application/octet-stream"];
        }
    }
    // RUFFLE_RES must be pre-defined and it is used in dynSrc.js so it can be mangled
    return `const RUFFLE_RES = ${JSON.stringify(generated)};`;
}

function patchPlayerJs(playerJsContent) {
    // patch import() since import is a keyword
    const importRegex = /import\(([`"'][a-zA-Z0-9\-_/\.]*[`"'])\)/g;
    return playerJsContent.replace(importRegex, (_, path) => `import(window.RufflePlayerDynSrc(${path}))`);
}

function generatePlayerJs(dynSrcJsPath, playerJsPath, dynSrcContent) {
    const dynSrcJsContent = readFileSync(dynSrcJsPath, "utf-8");
    const playerJsContent = readFileSync(playerJsPath, "utf-8");
    return minifyTerser(`(() => { \n${dynSrcContent}\n${dynSrcJsContent}\n${patchPlayerJs(playerJsContent)}\n})();`, {
        mangle: true,
        compress: true,
    }).code;
}

function replaceJs(jsContent) {
    // replace it so html will work
    return jsContent.replace(/(<)(\/script>)/g, '\\x3C$2');
}

function inlineJsToHtml(htmlContent, jsContent) {
    return htmlContent.replace('<script src="ruffle.js"></script>', () => `<script>${jsContent}</script>`);
}

function minimizeHtml(htmlContent) {
    return minifyHtml(htmlContent, {
        collapseWhitespace: true,
        removeComments: true,
        minifyCSS: true,
        minifyJS: true,
    });
}

const DIST_DIR = "dist-local";

function getResName(path) {
    const slashIndex = path.indexOf("/");
    if (slashIndex >= 0) {
        // leave filename only
        path = path.substring(slashIndex + 1);
    }
    return path;
}

function readBase64(path) {
    const suffix = path.substring(path.lastIndexOf("."));
    const mime = suffixToMime[suffix] || "application/octet-stream";
    return `data:${mime};base64,${readFileSync(`${DIST_DIR}/${path}`).toString("base64")}`;
}

function patchCss(cssContent) {
    const urlRegex = /url\(([a-zA-Z0-9\-_/\.]*)\)/g;
    return cssContent.replace(urlRegex, (_, path) => `url(${readBase64(path)})`);
}

function patchIndexHtml(htmlContent, indexJsContent) {
    // patch existing resources
    const htmlDom = new JSDOM(htmlContent, {
        contentType: "text/html",
    });
    const document = htmlDom.window.document;
    for (const scriptElement of document.getElementsByTagName("script")) {
        const src = scriptElement.getAttribute("src");
        if (src && getResName(src) == "index.js") {
            scriptElement.removeAttribute("src");
            scriptElement.innerHTML = indexJsContent;
        }
    }
    for (const linkElement of document.getElementsByTagName("link")) {
        const href = linkElement.getAttribute("href");
        if (href) {
            const rel = linkElement.getAttribute("rel");
            if (rel == "icon") {
                linkElement.setAttribute("href", readBase64(href));
            } else if (rel == "stylesheet") {
                const patchedCss = patchCss(readFileSync(`${DIST_DIR}/${href}`, "utf-8"));
                const styleElement = document.createElement("style");
                styleElement.innerHTML = patchedCss;
                linkElement.insertAdjacentElement("beforebegin", styleElement);
                linkElement.remove();
            }
        }
    }
    return document.documentElement.outerHTML;
}

async function main() {
    console.log("Packaging...");
    writeFileSync(`${DIST_DIR}/swfs.json`, JSON.stringify({
        "swfs": [
            {
                "location": "logo-anim.swf",
                "title": "Ruffle Logo",
                "author": "Ruffle contributors",
                "authorLink": "https://ruffle.rs",
                "config": {
                    "autoplay": "on",
                    "unmuteOverlay": "hidden"
                }
            }
        ]
    }), "utf-8");
    const dynSrcContent = generateDynSrcContent(DIST_DIR, /^((asset-|chunk-)(ruffle_web.*\.js|.+\.wasm)|.+\.swf|.+\.svg|swfs\.json)$/);
    const indexJs = replaceJs(generatePlayerJs(`${DIST_DIR}/url-patcher.unpacked.js`, `${DIST_DIR}/index.js`, dynSrcContent));
    let htmlContent = readFileSync(`${DIST_DIR}/index.html`, "utf-8");
    htmlContent = await minimizeHtml(patchIndexHtml(htmlContent, indexJs));
    writeFileSync(`${DIST_DIR}/RuffleDemoSA.html`, htmlContent, "utf-8");
}

main();
