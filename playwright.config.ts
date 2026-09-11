import { defineConfig, devices } from '@playwright/test';

const basePath = process.env.BASE_PATH || '/type-inference-demo/';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: 'list',
  use: {
    baseURL: `http://127.0.0.1:4173${basePath}`,
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npm run build && npm run preview -- --host 127.0.0.1 --port 4173 --strictPort',
    env: { BASE_PATH: basePath },
    url: `http://127.0.0.1:4173${basePath}`,
    reuseExistingServer: false,
    timeout: 180_000,
  },
});
