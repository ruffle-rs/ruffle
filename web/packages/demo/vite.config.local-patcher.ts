import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";

// https://vitejs.dev/config/
export default defineConfig({
    build: {
        outDir: "dist-local",
        emptyOutDir: false,
        lib: {
            entry: fileURLToPath(
                new URL("./src/local-patcher/url-patcher.js", import.meta.url),
            ),
            formats: ["es"],
            fileName: () => "url-patcher.unpacked.js",
        },
        rollupOptions: {
            external: [],
            output: {
                inlineDynamicImports: true,
            },
        },
        minify: "esbuild",
        target: "es2018",
        sourcemap: true,
    },
});
