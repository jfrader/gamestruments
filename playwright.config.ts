import { defineConfig, devices } from "@playwright/test";

const productionBaseUrl = process.env.PLAYWRIGHT_BASE_URL;

export default defineConfig({
  testDir: "tests/browser",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: "line",
  use: {
    baseURL: productionBaseUrl ?? "http://127.0.0.1:58317",
    headless: true,
    reducedMotion: "no-preference",
  },
  projects: [
    {
      name: "chromium",
      testIgnore: /firefox-animation-compat\.spec\.ts/,
      use: devices["Desktop Chrome"],
    },
    {
      name: "firefox",
      testMatch: /firefox-animation-compat\.spec\.ts/,
      use: devices["Desktop Firefox"],
    },
  ],
  webServer: productionBaseUrl
    ? undefined
    : {
        command: "npm run dev -- --port 58317 --strictPort",
        url: "http://127.0.0.1:58317/",
        reuseExistingServer: !process.env.CI,
      },
});
