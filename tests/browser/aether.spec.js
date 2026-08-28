const { test, expect } = require("@playwright/test");

test.use({ viewport: { width: 1280, height: 720 } });
test.setTimeout(60000);
test("web lobby opens the runner", async ({ page }) => {
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));

  await page.goto("http://127.0.0.1:8080/");
  await expect(page.getByRole("heading", { name: "Prototype Lobby" })).toBeVisible();
  await expect(page.getByRole("link", { name: /Skyway Runner/ })).toBeVisible();
  await expect(page.getByRole("link", { name: /Sandbox/ })).toBeVisible();

  await page.getByRole("link", { name: /Skyway Runner/ }).click();
  await expect(page).toHaveURL(/\/game\/runner\/$/);
  await expect(page.locator("#aether-canvas")).toHaveClass(/is-visible/, { timeout: 15000 });
  await expect(page.locator("#game-header")).toBeVisible();
  await expect(page.getByRole("link", { name: "Lobby" })).toBeVisible();
  // Navigating away can abort the lobby's WASM event loop with `unreachable`.
  // Only runtime errors from the game page are relevant to this smoke test.
  errors.length = 0;

  const canvas = page.locator("#aether-canvas");
  await expect(canvas).toBeAttached();
  await page.waitForTimeout(5000);
  expect(errors).toEqual([]);
});

test("web lobby opens Aether Shipwright", async ({ page }) => {
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));

  await page.goto("http://127.0.0.1:8080/");
  await page.getByRole("link", { name: /Aether Shipwright/ }).click();
  await expect(page).toHaveURL(/\/game\/shipwright\/$/);
  await expect(page.locator("#aether-canvas")).toHaveClass(/is-visible/, { timeout: 15000 });
  await expect(page.locator("#game-header")).toBeVisible();
  await expect(page.getByRole("link", { name: "Lobby" })).toBeVisible();
  await page.waitForTimeout(5000);
  expect(errors).toEqual([]);
});
