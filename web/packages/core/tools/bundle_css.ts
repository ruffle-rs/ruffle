import { replaceInFileSync } from "replace-in-file";
import fs from "fs";

const css = fs
    .readFileSync("src/internal/ui/static-styles.css", "utf8")
    .replaceAll("\r", "");

replaceInFileSync({
    files: "dist/**",
    from: [/"\s*\/\*\s*%STATIC_STYLES_CSS%\s*\*\/\s*"/g],
    to: [JSON.stringify(css)],
});
