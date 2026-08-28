import type { Page, TestInfo } from "@playwright/test";

export async function setForwardedClient(page: Page, testInfo: TestInfo) {
  let hash = 0;
  for (const character of testInfo.titlePath.join("/")) {
    hash = (hash * 31 + character.charCodeAt(0)) & 0xffff;
  }
  const thirdOctet = (hash >> 8) || 1;
  const fourthOctet = (hash & 0xff) || 1;
  await page.setExtraHTTPHeaders({
    "X-Forwarded-For": `198.18.${thirdOctet}.${fourthOctet}`,
  });
}
