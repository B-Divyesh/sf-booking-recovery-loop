export type SiteRoute = "home" | "demo" | "privacy" | "terms" | "not-found";

export interface PageMeta {
  readonly title: string;
  readonly description: string;
  readonly heading: string;
  readonly canonicalPath: string;
}

const pages: Readonly<Record<SiteRoute, PageMeta>> = {
  home: {
    title: "Booking Recovery Loop — recover paid sessions",
    description: "See where a paid booking stopped, check consent, and run a safe sample follow-up with a delivery receipt.",
    heading: "Recover paid sessions before they disappear",
    canonicalPath: "/"
  },
  demo: {
    title: "Demo — Booking Recovery Loop",
    description: "Try a consent-aware booking recovery with isolated sample data and a simulated delivery receipt.",
    heading: "Recover a sample booking",
    canonicalPath: "/demo"
  },
  privacy: {
    title: "Privacy — Booking Recovery Loop",
    description: "How Booking Recovery Loop handles temporary demo data and protects personal information.",
    heading: "Your sample stays separate",
    canonicalPath: "/privacy"
  },
  terms: {
    title: "Terms — Booking Recovery Loop",
    description: "The terms for using the Booking Recovery Loop sample workspace and future paid service.",
    heading: "Terms for the sample workspace",
    canonicalPath: "/terms"
  },
  "not-found": {
    title: "Page not found — Booking Recovery Loop",
    description: "The requested Booking Recovery Loop page was not found.",
    heading: "That page is not here",
    canonicalPath: "/404"
  }
};

export function routeFor(pathname: string, search = ""): SiteRoute {
  if (pathname === "/" && new URLSearchParams(search).get("demo") === "1") {
    return "demo";
  }

  switch (pathname) {
    case "/":
      return "home";
    case "/demo":
      return "demo";
    case "/privacy":
      return "privacy";
    case "/terms":
      return "terms";
    case "/404":
      return "not-found";
    default:
      return "not-found";
  }
}

export function pageFor(pathname: string, search = ""): PageMeta {
  return pages[routeFor(pathname, search)];
}

export function canonicalUrl(path: string): string {
  return new URL(path, "https://booking-recovery-loop.sociobot.in").toString();
}
