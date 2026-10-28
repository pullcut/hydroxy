import { expect, test } from "@playwright/test";

test("zooming one line chart moves the other", async ({ page }) => {
  await page.goto("/charts/linked-charts");
  const demo = page.locator("#example");
  await expect(demo).toBeVisible();
  await page.waitForLoadState("networkidle");

  const a = demo.getByTestId("linked-a");
  const b = demo.getByTestId("linked-b");
  const labels = (chart: typeof a) => chart.locator('[data-part="x-label"]').allTextContents();
  const before = await labels(b);

  const plot = a.locator('[data-part="overlay"]');
  const box = await plot.boundingBox();
  await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
  await page.mouse.wheel(0, -600);

  await expect.poll(() => labels(b)).not.toEqual(before);
  expect(await labels(b)).toEqual(await labels(a));
});

test("clicking a bar filters the table", async ({ page }) => {
  await page.goto("/charts/linked-charts");
  const demo = page.locator("#example");
  await expect(demo).toBeVisible();
  await page.waitForLoadState("networkidle");

  const table = demo.getByTestId("linked-table");
  await expect(table.locator("tr")).toHaveCount(12);
  await demo.locator('[data-part="column"]').nth(1).click();
  await expect(table.locator("tr")).toHaveCount(3);
  await expect(table.locator("caption")).toHaveText("Support tickets");
});
