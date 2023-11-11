import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      "@rogueBoi": fileURLToPath(new URL("./src/lib", import.meta.url))
    }
  }
});
