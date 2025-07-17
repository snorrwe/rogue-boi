import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  base: "",
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: {
      "@rogueBoi": fileURLToPath(new URL("./src/lib", import.meta.url))
    }
  }
});
