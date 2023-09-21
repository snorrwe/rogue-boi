import svelte from "rollup-plugin-svelte";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import livereload from "rollup-plugin-livereload";
import terser from "@rollup/plugin-terser";
import rust from "@wasm-tool/rollup-plugin-rust";
import alias from "@rollup/plugin-alias";
import path from "node:path";

const shouldWatch = process.env.ROLLUP_WATCH;
const production = process.env.BUILD === "production";
const projectRootDir = path.resolve(__dirname);

export default [
  {
    input: "ui/main.js",
    output: {
      sourcemap: true,
      format: "iife",
      name: "rogue",
      file: "public/build/bundle.js"
    },
    plugins: [
      alias({
        entries: [{ find: "@rogueBoi", replacement: path.resolve(projectRootDir, "ui/lib") }]
      }),
      svelte({
        compilerOptions: {
          // enable run-time checks when not in production
          dev: !production,
          css: true
        },
        emitCss: false
      }),

      // If you have external dependencies installed from
      // npm, you'll most likely need these plugins. In
      // some cases you'll need additional configuration -
      // consult the documentation for details:
      // https://github.com/rollup/plugins/tree/master/packages/commonjs
      resolve({
        browser: true,
        dedupe: ["svelte"]
      }),
      commonjs(),
      rust({
        debug: !production,
        verbose: true,
        serverPath: "/build/",
        watchPatterns: ["src/**", "assets/**"]
      }),
      icons(),

      // In dev mode, call `npm run start` once
      // the bundle has been generated
      shouldWatch && serve(),

      // Watch the `public` directory and refresh the
      // browser on changes when not in production
      shouldWatch && livereload("public"),

      // If we're building for production (npm run build
      // instead of npm run dev), minify
      production && terser(),
      production && bundle()
    ],
    watch: {
      clearScreen: false
    }
  }
];

function icons() {
  return {
    buildStart() {
      require("child_process").spawn("cargo", ["xtask", "copy-icons"], {
        stdio: ["ignore", "inherit", "inherit"],
        shell: true
      });
    }
  };
}

function serve() {
  let started = false;

  return {
    writeBundle() {
      if (!started) {
        started = true;

        require("child_process").spawn("npm", ["run", "start", "--", "--dev", "-i"], {
          stdio: ["ignore", "inherit", "inherit"],
          shell: true
        });
      }
    }
  };
}

function bundle() {
  return {
    name: "build-bundle",
    writeBundle() {
      require("child_process").spawn("cargo", ["xtask", "bundle"], {
        stdio: ["ignore", "inherit", "inherit"],
        shell: true
      });
    }
  };
}
