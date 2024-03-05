module.exports = {
  root: true,
  extends: ["prettier", "plugin:svelte/recommended"],
  plugins: [],
  overrides: [],
  parserOptions: {
    sourceType: "module",
    ecmaVersion: 2020
  },
  settings: {
    svelte: {
      ignoreWarnings: ["svelte/no-at-html-tags", "svelte/valid-compile"]
    }
  },
  env: {
    browser: true,
    es2017: true,
    node: true
  }
};
