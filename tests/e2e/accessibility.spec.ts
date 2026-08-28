import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const routes = [
  ["/", "Booking Recovery Loop — recover paid sessions"],
  ["/demo", "Demo — Booking Recovery Loop"],
  ["/privacy", "Privacy — Booking Recovery Loop"],
  ["/terms", "Terms — Booking Recovery Loop"],
  ["/missing-page", "Page not found — Booking Recovery Loop"]
] as const;

for (const [path, title] of routes) {
  test(`${path} has route metadata and no serious accessibility violations`, async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    await page.goto(path);
    if (path === "/demo") {
      await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
    }
    await expect(page).toHaveTitle(title);
    await expect(page.locator("main")).toHaveCount(1);
    await expect(page.locator("h1")).toHaveCount(1);
    await expect(page.locator('img:not([alt])')).toHaveCount(0);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
    expect(errors).toEqual([]);
  });
}

test("keyboard users can skip, select, and run the recovery", async ({ page }) => {
  await openDemo(page);
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to main content" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("main")).toBeFocused();

  const maya = page.locator('[data-action="select-attempt"]').filter({ hasText: "Maya Patel" });
  await maya.focus();
  await page.keyboard.press("Space");
  const recovery = page.getByRole("button", { name: "Run sample follow-up" });
  await recovery.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".receipt-timeline li")).toContainText("Delivered");
});

async function openDemo(page: import("@playwright/test").Page) {
  await page.goto("/?demo=1");
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
}
