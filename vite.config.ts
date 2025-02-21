import deno from "@deno/vite-plugin";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const cwd = dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig({
    root: resolve(cwd, "client"),
    plugins: [deno()],
    build: {
        target: "esnext",
        outDir: resolve(cwd, "dist"),
        emptyOutDir: true,
        rollupOptions: {
            input: {
                index: resolve(cwd, "client/index.html"),
                login: resolve(cwd, "client/login.html"),
                me: resolve(cwd, "client/me.html"),
                signup: resolve(cwd, "client/signup.html"),
            },
        },
    },
});
