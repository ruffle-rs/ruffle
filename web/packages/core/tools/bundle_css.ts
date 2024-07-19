import { replaceInFileSync } from "replace-in-file";
import fs from "fs";
import postcss from "postcss";
import cssnanoPlugin from "cssnano";

const originalCss = fs
    .readFileSync("src/internal/ui/static-styles.css", "utf8")
    .replaceAll("\r", "");

const processor = postcss([
    cssnanoPlugin({
        preset: ["advanced", { autoprefixer: { add: true } }],
    }),
]);
processor
    .process(originalCss, { from: "src/internal/ui/static-styles.css" })
    .then((result) => {
        replaceInFileSync({
            files: "dist/**",
            from: [/"\s*\/\*\s*%STATIC_STYLES_CSS%\s*\*\/\s*"/g],
            to: [JSON.stringify(result.css)],
        });
    });
