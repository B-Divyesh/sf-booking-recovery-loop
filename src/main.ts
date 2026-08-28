import "./styles/tokens.css";
import "./styles/app.css";

import { canonicalUrl, pageFor, routeFor, type SiteRoute } from "./lib/site";

const applicationRoot = document.querySelector<HTMLDivElement>("#app");

if (!applicationRoot) {
  throw new Error("The application root is missing.");
}

const app: HTMLDivElement = applicationRoot;

function contentFor(route: SiteRoute): string {
  switch (route) {
    case "home":
      return `
        <section class="foundation-lede" aria-describedby="foundation-status">
          <p class="eyebrow">Engineering foundation</p>
          <h1 tabindex="-1">Booking Recovery Loop foundation</h1>
          <p id="foundation-status" class="lede">The venture plan, visual system, and build tooling are ready for M1.</p>
          <a class="button button-primary" href="/demo">Open the planned demo route</a>
        </section>
        <section class="foundation-grid" aria-labelledby="foundation-next">
          <div class="rail-mark" aria-hidden="true"><span></span><span></span><span></span></div>
          <div>
            <h2 id="foundation-next">What M1 adds</h2>
            <p>A safe sample recovery loop, plain-language public pages, and claim-level browser tests.</p>
          </div>
        </section>`;
    case "demo":
      return `
        <section class="foundation-lede">
          <p class="eyebrow">Planned M1 sandbox</p>
          <h1 tabindex="-1">Demo foundation</h1>
          <p class="lede">M1 will place isolated sample data here. This scaffold does not store bookings or send messages.</p>
          <a class="button button-secondary" href="/">Back to the foundation</a>
        </section>`;
    case "privacy":
      return `
        <section class="foundation-lede prose">
          <p class="eyebrow">Planned public page</p>
          <h1 tabindex="-1">Privacy foundation</h1>
          <p>This pre-product scaffold does not collect or store customer data.</p>
          <p>M1 will replace this placeholder with the product’s plain-language privacy page before any data feature ships.</p>
        </section>`;
    case "terms":
      return `
        <section class="foundation-lede prose">
          <p class="eyebrow">Planned public page</p>
          <h1 tabindex="-1">Terms foundation</h1>
          <p>This pre-product scaffold does not take payment or offer a customer service.</p>
          <p>M1 will replace this placeholder with terms that match the product before any billing or data feature ships.</p>
        </section>`;
    case "not-found":
      return `
        <section class="foundation-lede prose">
          <p class="eyebrow">404</p>
          <h1 tabindex="-1">That page is not here</h1>
          <p>Try the foundation home instead.</p>
          <a class="button button-primary" href="/">Go to the foundation</a>
        </section>`;
  }
}

function navigation(currentRoute: SiteRoute): string {
  const links: ReadonlyArray<readonly [string, SiteRoute, string]> = [
    ["/demo", "demo", "Demo"],
    ["/privacy", "privacy", "Privacy"],
    ["/terms", "terms", "Terms"]
  ];

  return links
    .map(
      ([href, route, label]) =>
        `<a href="${href}"${currentRoute === route ? ' aria-current="page"' : ""}>${label}</a>`
    )
    .join("");
}

function setDocumentMetadata(pathname: string, search: string): void {
  const page = pageFor(pathname, search);
  document.title = page.title;
  document
    .querySelector<HTMLMetaElement>('meta[name="description"]')
    ?.setAttribute("content", page.description);
  document
    .querySelector<HTMLLinkElement>('link[rel="canonical"]')
    ?.setAttribute("href", canonicalUrl(page.canonicalPath));
}

function render({ focusHeading }: { focusHeading: boolean }): void {
  const route = routeFor(window.location.pathname, window.location.search);
  setDocumentMetadata(window.location.pathname, window.location.search);
  app.innerHTML = `
    <a class="skip-link" href="#main">Skip to main content</a>
    <header class="site-header">
      <a class="wordmark" href="/" aria-label="Booking Recovery Loop home">Booking Recovery Loop</a>
      <nav aria-label="Primary navigation">${navigation(route)}</nav>
    </header>
    <main id="main">${contentFor(route)}</main>
    <footer class="site-footer">
      <p>Booking Recovery Loop helps small practices protect paid appointments.</p>
      <div><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built by Param Factory · foundation</span></div>
    </footer>
    <p class="sr-only" aria-live="polite" aria-atomic="true" id="route-announcement">${pageFor(window.location.pathname, window.location.search).heading}</p>`;

  if (focusHeading) {
    document.querySelector<HTMLElement>("main h1")?.focus();
  }
}

function isInternalNavigation(event: MouseEvent): event is MouseEvent & { currentTarget: HTMLAnchorElement } {
  const target = event.target;
  if (!(target instanceof Element)) {
    return false;
  }

  const anchor = target.closest<HTMLAnchorElement>("a[href]");
  if (!anchor || event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
    return false;
  }

  const destination = new URL(anchor.href, window.location.href);
  return destination.origin === window.location.origin;
}

document.addEventListener("click", (event) => {
  if (!isInternalNavigation(event)) {
    return;
  }

  const anchor = (event.target as Element).closest<HTMLAnchorElement>("a[href]");
  if (!anchor) {
    return;
  }

  const destination = new URL(anchor.href, window.location.href);
  event.preventDefault();
  window.history.pushState({}, "", `${destination.pathname}${destination.search}`);
  render({ focusHeading: true });
});

window.addEventListener("popstate", () => render({ focusHeading: true }));
render({ focusHeading: false });
