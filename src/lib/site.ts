export type SiteRoute = "home" | "demo" | "start" | "app" | "data" | "booking" | "complete" | "privacy" | "terms" | "not-found";

export interface PageMeta {
  readonly title: string;
  readonly description: string;
  readonly heading: string;
  readonly canonicalPath: string;
}

const pages: Readonly<Record<SiteRoute, PageMeta>> = {
  home: {
    title: "Booking Recovery Loop — recover unfinished bookings",
    description: "Recover stopped paid bookings with recorded consent, hosted payment handoff, and provider delivery receipts.",
    heading: "Recover unfinished paid-session bookings",
    canonicalPath: "/"
  },
  demo: {
    title: "Demo — Booking Recovery Loop",
    description: "Try a consent-aware booking recovery with isolated sample data and a simulated delivery receipt.",
    heading: "Recover a sample booking",
    canonicalPath: "/demo"
  },
  start: {
    title: "Start a practice — Booking Recovery Loop",
    description: "Create a private practice workspace and publish one paid-session booking page.",
    heading: "Set up your booking recovery page",
    canonicalPath: "/start"
  },
  app: {
    title: "Recovery queue — Booking Recovery Loop",
    description: "Review booking attempts, consent records, and delivery receipts for your practice.",
    heading: "Review bookings that need action",
    canonicalPath: "/app"
  },
  data: {
    title: "Data controls — Booking Recovery Loop",
    description: "Export or delete your Booking Recovery Loop practice data.",
    heading: "Export or delete practice data",
    canonicalPath: "/app/settings/data"
  },
  booking: {
    title: "Book a paid session — Booking Recovery Loop",
    description: "Choose a session, record contact permission, and continue to hosted payment.",
    heading: "Finish your paid session booking",
    canonicalPath: "/b"
  },
  complete: {
    title: "Payment check — Booking Recovery Loop",
    description: "Your booking remains pending until the hosted payment provider confirms the deposit.",
    heading: "Your deposit is being checked",
    canonicalPath: "/b"
  },
  privacy: {
    title: "Privacy — Booking Recovery Loop",
    description: "How Booking Recovery Loop handles temporary demo data and protects personal information.",
    heading: "Control your booking data",
    canonicalPath: "/privacy"
  },
  terms: {
    title: "Terms — Booking Recovery Loop",
    description: "The terms for using the Booking Recovery Loop sample workspace and future paid service.",
    heading: "Terms for using Booking Recovery Loop",
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
    case "/start":
      return "start";
    case "/app":
      return "app";
    case "/app/settings/data":
      return "data";
    case "/privacy":
      return "privacy";
    case "/terms":
      return "terms";
    case "/404":
      return "not-found";
    default:
      if (/^\/b\/[a-z0-9-]+\/complete$/.test(pathname)) return "complete";
      if (/^\/b\/[a-z0-9-]+$/.test(pathname)) return "booking";
      return "not-found";
  }
}

export function pageFor(pathname: string, search = ""): PageMeta {
  return pages[routeFor(pathname, search)];
}

export function canonicalUrl(path: string): string {
  return new URL(path, "https://booking-recovery-loop.sociobot.in").toString();
}
