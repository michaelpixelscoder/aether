const { test, expect } = require("@playwright/test");

test.use({ viewport: { width: 1280, height: 720 } });
test("web lobby opens the sandbox", async ({ page }) => {
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));

  await page.goto("http://127.0.0.1:8080/");
  await expect(page.getByRole("heading", { name: "Prototype Lobby" })).toBeVisible();
  await expect(page.getByRole("link", { name: /Sandbox/ })).toBeVisible();

  await page.getByRole("link", { name: /Sandbox/ }).click();
  await expect(page).toHaveURL(/\/game\/sandbox\/$/);

  const canvas = page.locator("#aether-canvas");
  await expect(canvas).toBeAttached();
  await page.waitForTimeout(5000);
  expect(errors).toEqual([]);
});
