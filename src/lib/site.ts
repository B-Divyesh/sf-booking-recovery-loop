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
    description: "Booking Recovery Loop engineering foundation.",
    heading: "Booking Recovery Loop foundation",
    canonicalPath: "/"
  },
  demo: {
    title: "Demo — Booking Recovery Loop",
    description: "The planned isolated sample workspace for Booking Recovery Loop.",
    heading: "Demo foundation",
    canonicalPath: "/demo"
  },
  privacy: {
    title: "Privacy — Booking Recovery Loop",
    description: "Privacy information for Booking Recovery Loop.",
    heading: "Privacy foundation",
    canonicalPath: "/privacy"
  },
  terms: {
    title: "Terms — Booking Recovery Loop",
    description: "Terms information for Booking Recovery Loop.",
    heading: "Terms foundation",
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
