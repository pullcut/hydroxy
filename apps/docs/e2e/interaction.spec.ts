import { expect, Locator, Page, test } from "@playwright/test";

/** Open a page and return its live example, once the app has hydrated. */
async function example(page: Page, path: string) {
  await page.goto(path);
  const demo = page.locator("#example");
  await expect(demo).toBeVisible();
  // Demos become interactive a moment after first paint.
  await page.waitForLoadState("networkidle");
  return demo;
}

test("combobox filters and picks with the keyboard", async ({ page }) => {
  const demo = await example(page, "/components/combobox");
  const input = demo.getByRole("combobox").first();
  await input.click();
  await expect(demo.getByRole("listbox").first()).toBeVisible();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(input).not.toHaveValue("");
  await page.keyboard.press("Escape");
});

test("menu opens, moves and closes with the keyboard", async ({ page }) => {
  const demo = await example(page, "/components/menu");
  const trigger = demo.getByRole("button").first();
  await trigger.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("menu").first()).toBeVisible();
  await page.keyboard.press("ArrowDown");
  await expect(page.getByRole("menuitem").first()).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("menu")).toHaveCount(0);
  await expect(trigger).toBeFocused();
});

test("dialog traps focus and returns it on close", async ({ page }) => {
  const demo = await example(page, "/components/dialog");
  const trigger = demo.getByRole("button").first();
  await trigger.click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  for (let i = 0; i < 6; i++) {
    await page.keyboard.press("Tab");
    expect(await dialog.evaluate((d) => d.contains(document.activeElement))).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(trigger).toBeFocused();
});

test("date picker opens a grid and picks a day", async ({ page }) => {
  const demo = await example(page, "/components/date-picker");
  await demo.getByRole("button").first().click();
  const grid = page.getByRole("grid").first();
  await expect(grid).toBeVisible();
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("Enter");
  await expect(grid).toHaveCount(0);
});

test("spreadsheet grid edits a cell and recalculates a formula", async ({ page }) => {
  const demo = await example(page, "/components/spreadsheet-grid");
  const grid = demo.getByRole("grid").first();
  await expect(grid).toBeVisible();
  const cell = grid.getByRole("gridcell").first();
  await cell.click();
  await page.keyboard.press("F2");
  await page.keyboard.type("=1+2");
  await page.keyboard.press("Enter");
  await expect(grid.getByRole("gridcell", { name: "3", exact: true }).first()).toBeVisible();
});

test("spreadsheet grid pastes tab separated text", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  const demo = await example(page, "/components/spreadsheet-grid");
  const grid = demo.getByRole("grid").first();
  await grid.getByRole("gridcell").first().click();
  await page.evaluate(() => navigator.clipboard.writeText("7\t8\n9\t10"));
  await page.keyboard.press("ControlOrMeta+v");
  await expect(grid.getByRole("gridcell", { name: "10", exact: true }).first()).toBeVisible();
});

test("node editor undoes a change", async ({ page }) => {
  const demo = await example(page, "/components/node-editor");
  const nodes = demo.locator("[data-part~='node']");
  const before = await nodes.count();
  expect(before).toBeGreaterThan(0);
  await nodes.first().click();
  await page.keyboard.press("Delete");
  await expect(nodes).toHaveCount(before - 1);
  await page.keyboard.press("ControlOrMeta+z");
  await expect(nodes).toHaveCount(before);
});

test("node editor connects two ports and undoes it", async ({ page }) => {
  const demo = await example(page, "/components/node-editor");
  await demo.getByRole("button", { name: "Zoom to fit" }).click();
  const edges = demo.locator("[data-part~='edge']");
  const before = await edges.count();
  // "Pick fields" Email (text) is free to feed "Post to #sales" Message (text).
  const out = demo.locator("[data-part~='port'][data-side='output']").filter({ hasText: /Email/ }).first();
  const dest = demo.locator("[data-part~='port'][data-side='input']").filter({ hasText: /Message/ }).first();
  const a = (await out.boundingBox())!;
  const b = (await dest.boundingBox())!;
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 8 });
  await page.mouse.up();
  await expect(edges).toHaveCount(before + 1);
  await page.keyboard.press("ControlOrMeta+z");
  await expect(edges).toHaveCount(before);
});

test("whiteboard draws a stroke and undoes it", async ({ page }) => {
  const demo = await example(page, "/components/whiteboard");
  const surface = demo.locator("[data-part~='viewport']").first();
  const box = (await surface.boundingBox())!;
  const marks = demo.locator("[data-part~='shape']");
  const before = await marks.count();
  await demo.getByRole("button", { name: /pen/i }).first().click();
  await page.mouse.move(box.x + 60, box.y + 60);
  await page.mouse.down();
  await page.mouse.move(box.x + 140, box.y + 120, { steps: 6 });
  await page.mouse.up();
  await expect(marks).toHaveCount(before + 1);
  await page.keyboard.press("ControlOrMeta+z");
  await expect(marks).toHaveCount(before);
});

/** Pointer position inside the scheduler's visible scroll area, `dy` below its top. */
async function schedulerPoint(demo: Locator, x: number, dy: number) {
  const scroller = (await demo.locator("[data-part~='scroller']").first().boundingBox())!;
  return { x, y: scroller.y + dy };
}

test("week scheduler creates an event by dragging", async ({ page }) => {
  const demo = await example(page, "/components/week-scheduler");
  const events = demo.locator("[data-part~='event']");
  const before = await events.count();
  // The last column is the weekend, which has no events to hit.
  const column = demo.locator("[data-part~='column']").last();
  await column.scrollIntoViewIfNeeded();
  const col = (await column.boundingBox())!;
  const start = await schedulerPoint(demo, col.x + col.width / 2, 120);
  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x, start.y + 60, { steps: 5 });
  await page.mouse.up();
  await expect(events).toHaveCount(before + 1);
});

test("week scheduler moves an event by dragging", async ({ page }) => {
  const demo = await example(page, "/components/week-scheduler");
  const event = demo.locator("[data-part~='event']").filter({ hasText: "Sprint planning" }).first();
  await event.scrollIntoViewIfNeeded();
  const time = event.locator("[data-part~='event-time']");
  const before = await time.textContent();
  const box = (await event.boundingBox())!;
  const x = box.x + box.width / 2;
  const y = box.y + Math.min(box.height / 2, 30);
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x, y + 80, { steps: 8 });
  await page.mouse.up();
  const moved = demo.locator("[data-part~='event']").filter({ hasText: "Sprint planning" }).first();
  await expect(moved.locator("[data-part~='event-time']")).not.toHaveText(before ?? "");
});

test("virtual list jumps to row 50,000 from the keyboard", async ({ page }) => {
  const demo = await example(page, "/components/virtual-list");
  const list = demo.getByRole("listbox").first();
  await list.focus();
  await page.keyboard.press("Home");
  await page.keyboard.press("End");
  // The active row is the last of a set with ~100,000 rows, far past anything rendered.
  const active = list.locator("[data-active]");
  await expect(active).toHaveCount(1);
  const pos = Number(await active.getAttribute("aria-posinset"));
  const size = Number(await active.getAttribute("aria-setsize"));
  expect(size).toBeGreaterThan(50_000);
  expect(pos).toBe(size);
  // Rendering stays windowed however long the list is.
  expect(await demo.getByRole("option").count()).toBeLessThan(200);
});

test("mention input inserts and deletes a mention", async ({ page }) => {
  const demo = await example(page, "/components/mention-input");
  const input = demo.getByRole("combobox").or(demo.getByRole("textbox")).first();
  await input.click();
  await page.keyboard.type("hi @An");
  await expect(page.getByRole("option", { name: /Ana Lima/ }).first()).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(input).toContainText("Ana Lima");
  await page.keyboard.press("Backspace");
  await expect(input).not.toContainText("Ana Lima");
});

test("bar race plays and scrubs", async ({ page }) => {
  const demo = await example(page, "/charts/bar-race-chart");
  const play = demo.getByRole("button", { name: /play|pause/i }).first();
  await expect(play).toBeVisible();
  const scrub = demo.getByRole("slider").first();
  const before = await scrub.getAttribute("aria-valuenow");
  await scrub.focus();
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await expect(scrub).not.toHaveAttribute("aria-valuenow", before ?? "");
});
