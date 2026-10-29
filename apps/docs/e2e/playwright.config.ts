import { defineConfig } from "@playwright/test";

// Starts the docs site with `cargo leptos serve` from apps/docs (port 8090), or reuses one
// already running locally. Set DOCS_URL to test another server instead.
export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  timeout: 30_000,
  use: { baseURL: process.env.DOCS_URL ?? "http://127.0.0.1:8090" },
  webServer: process.env.DOCS_URL
    ? undefined
    : {
        command: "cargo leptos serve",
        cwd: "..",
        url: "http://127.0.0.1:8090",
        reuseExistingServer: !process.env.CI,
        timeout: 600_000,
      },
});
