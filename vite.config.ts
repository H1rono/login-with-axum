import deno from "@deno/vite-plugin";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const cwd = dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig({
    root: resolve(cwd, "public"),
    plugins: [deno()],
    build: {
        target: "esnext",
        outDir: resolve(cwd, "dist"),
        emptyOutDir: true,
        rollupOptions: {
            input: {
                index: resolve(cwd, "public/index.html"),
                login: resolve(cwd, "public/login.html"),
                me: resolve(cwd, "public/me.html"),
                signup: resolve(cwd, "public/signup.html"),
            },
        },
    },
});
