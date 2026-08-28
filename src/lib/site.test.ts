import { describe, expect, it } from "vitest";

import { canonicalUrl, pageFor, routeFor } from "./site";

describe("site routes and metadata", () => {
  it("uses the demo route for the documented query entry point", () => {
    expect(routeFor("/", "?demo=1")).toBe("demo");
    expect(pageFor("/", "?demo=1").title).toBe("Demo — Booking Recovery Loop");
  });

  it("maps public policy routes to their own titles", () => {
    expect(pageFor("/privacy").title).toBe("Privacy — Booking Recovery Loop");
    expect(pageFor("/terms").title).toBe("Terms — Booking Recovery Loop");
  });

  it("gives unknown paths a usable not-found page", () => {
    expect(routeFor("/missing")).toBe("not-found");
    expect(pageFor("/missing").heading).toBe("That page is not here");
  });

  it("builds canonical URLs from an explicit path", () => {
    expect(canonicalUrl("/demo")).toBe(
      "https://booking-recovery-loop.sociobot.in/demo"
    );
  });

  it("uses a plain job as the landing heading", () => {
    expect(pageFor("/").heading).toBe("Recover paid sessions before they disappear");
    expect(pageFor("/").description.length).toBeLessThanOrEqual(155);
  });
});
