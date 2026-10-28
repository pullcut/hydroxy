import { defineConfig } from "@playwright/test";

// Serve the docs site first: `cargo leptos serve` from apps/docs (port 8090).
export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  timeout: 30_000,
  use: { baseURL: process.env.DOCS_URL ?? "http://127.0.0.1:8090" },
});
