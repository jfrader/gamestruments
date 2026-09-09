import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests/browser",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:58317",
    headless: true,
  },
  webServer: {
    command: "npm run dev -- --port 58317 --strictPort",
    url: "http://127.0.0.1:58317/",
    reuseExistingServer: !process.env.CI,
  },
});
