import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const THEMES = ["light", "dark"] as const;

/** Every docs page, read from the sidebar so new pages are covered without edits. */
async function pages(page: import("@playwright/test").Page): Promise<string[]> {
  const found = new Set<string>(["/", "/docs/getting-started", "/components", "/charts"]);
  for (const index of ["/components", "/charts"]) {
    await page.goto(index);
    const hrefs = await page.locator(`a[href^='${index}/']`).evaluateAll((as) =>
      as.map((a) => (a as HTMLAnchorElement).getAttribute("href")!),
    );
    hrefs.forEach((h) => found.add(h));
  }
  return [...found];
}

for (const theme of THEMES) {
  test(`no serious or critical axe violations in ${theme}`, async ({ page }) => {
    test.setTimeout(30 * 60_000);
    await page.addInitScript((t) => localStorage.setItem("ox-theme", t), theme);
    const failures: string[] = [];
    for (const path of await pages(page)) {
      await page.goto(path);
      await page.waitForLoadState("networkidle");
      const { violations } = await new AxeBuilder({ page }).analyze();
      for (const v of violations) {
        if (v.impact === "serious" || v.impact === "critical") {
          failures.push(`${path}: ${v.id} (${v.impact}) on ${v.nodes.length} node(s)`);
        }
      }
    }
    expect(failures, failures.join("\n")).toEqual([]);
  });
}
