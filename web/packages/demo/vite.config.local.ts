import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [react()],
    base: "",
    build: {
        outDir: "dist-local",
        rollupOptions: {
            input: ["./index.html"],
            output: {
                inlineDynamicImports: true,
                manualChunks: undefined,
                entryFileNames: "index.js",
                chunkFileNames: "chunk-[name].js",
                assetFileNames: "asset-[name][extname]",
            },
        },
    },
});
