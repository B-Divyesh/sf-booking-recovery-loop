(function(){const t=document.createElement("link").relList;if(t&&t.supports&&t.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))a(i);new MutationObserver(i=>{for(const n of i)if(n.type==="childList")for(const p of n.addedNodes)p.tagName==="LINK"&&p.rel==="modulepreload"&&a(p)}).observe(document,{childList:!0,subtree:!0});function o(i){const n={};return i.integrity&&(n.integrity=i.integrity),i.referrerPolicy&&(n.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?n.credentials="include":i.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function a(i){if(i.ep)return;i.ep=!0;const n=o(i);fetch(i.href,n)}})();const M="data:image/svg+xml,%3c!--%20Hand-made%20for%20Booking%20Recovery%20Loop,%20Param%20Factory,%202026;%20no%20external%20asset%20or%20model%20output.%20--%3e%3csvg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%20920%20620'%20role='img'%20aria-labelledby='title%20description'%3e%3ctitle%20id='title'%3eA%20booking%20recovery%20rail%3c/title%3e%3cdesc%20id='description'%3eA%20calm%20appointment%20rail%20showing%20one%20booking%20that%20needs%20a%20follow-up.%3c/desc%3e%3crect%20width='920'%20height='620'%20rx='44'%20fill='%23151E35'/%3e%3cpath%20d='M90%20397H830'%20stroke='%23536481'%20stroke-width='4'%20stroke-linecap='round'/%3e%3cpath%20d='M90%20413H830'%20stroke='%23202C49'%20stroke-width='2'%20stroke-dasharray='8%2014'/%3e%3ccircle%20cx='190'%20cy='397'%20r='15'%20fill='%2382E2C7'%20stroke='%230D1324'%20stroke-width='7'/%3e%3ccircle%20cx='460'%20cy='397'%20r='22'%20fill='%23FFBE5C'%20stroke='%230D1324'%20stroke-width='9'/%3e%3ccircle%20cx='730'%20cy='397'%20r='15'%20fill='%23202C49'%20stroke='%23536481'%20stroke-width='5'/%3e%3cg%20transform='translate(96%20122)'%3e%3cpath%20d='M18%200h188a18%2018%200%200%201%2018%2018v198a18%2018%200%200%201-18%2018H18A18%2018%200%200%201%200%20216V18A18%2018%200%200%201%2018%200Z'%20fill='%23202C49'%20stroke='%23536481'%20stroke-width='2'/%3e%3cpath%20d='M0%20160h224'%20stroke='%23536481'%20stroke-width='2'%20stroke-dasharray='7%208'/%3e%3ccircle%20cx='30'%20cy='38'%20r='9'%20fill='%2382E2C7'/%3e%3cpath%20d='M54%2030h120M30%2084h154M30%20116h104'%20stroke='%23CED4E5'%20stroke-width='11'%20stroke-linecap='round'%20opacity='.88'/%3e%3cpath%20d='M30%20190h92'%20stroke='%2382E2C7'%20stroke-width='10'%20stroke-linecap='round'/%3e%3c/g%3e%3cg%20transform='translate(348%2062)'%3e%3cpath%20d='M20%200h204a20%2020%200%200%201%2020%2020v258a20%2020%200%200%201-20%2020H20A20%2020%200%200%201%200%20278V20A20%2020%200%200%201%2020%200Z'%20fill='%23F8F3E8'/%3e%3cpath%20d='M0%20200h244'%20stroke='%23536481'%20stroke-width='2'%20stroke-dasharray='8%208'/%3e%3ccircle%20cx='36'%20cy='43'%20r='11'%20fill='%23FFBE5C'/%3e%3cpath%20d='M62%2034h122M36%2094h162M36%20132h129'%20stroke='%231E2434'%20stroke-width='12'%20stroke-linecap='round'%20opacity='.9'/%3e%3cpath%20d='M36%20169h82'%20stroke='%23536481'%20stroke-width='9'%20stroke-linecap='round'/%3e%3crect%20x='34'%20y='231'%20width='176'%20height='38'%20rx='10'%20fill='%23FFBE5C'/%3e%3cpath%20d='M67%20250h110'%20stroke='%231E2434'%20stroke-width='9'%20stroke-linecap='round'/%3e%3c/g%3e%3cg%20transform='translate(620%20150)'%3e%3cpath%20d='M18%200h188a18%2018%200%200%201%2018%2018v170a18%2018%200%200%201-18%2018H18A18%2018%200%200%201%200%20188V18A18%2018%200%200%201%2018%200Z'%20fill='%23202C49'%20stroke='%23536481'%20stroke-width='2'/%3e%3ccircle%20cx='30'%20cy='38'%20r='9'%20fill='%23536481'/%3e%3cpath%20d='M54%2030h120M30%2082h154M30%20114h104'%20stroke='%23CED4E5'%20stroke-width='11'%20stroke-linecap='round'%20opacity='.72'/%3e%3cpath%20d='M30%20158h104'%20stroke='%23536481'%20stroke-width='10'%20stroke-linecap='round'/%3e%3c/g%3e%3cpath%20d='M460%20432v90'%20stroke='%23FFBE5C'%20stroke-width='4'/%3e%3cpath%20d='M432%20522h56l-12%2024h-32Z'%20fill='%23FFBE5C'/%3e%3c/svg%3e",w="demo:workspace-token";class k extends Error{code;status;constructor(t,o,a){super(o),this.name="DemoApiError",this.code=t,this.status=a}}function L(e){return e.consent.email?e.state==="completed"?{allowed:!1,label:"Booking complete",explanation:"The booking is complete, so it does not need a follow-up."}:e.state==="recovered"?{allowed:!1,label:"Sample delivered",explanation:"The sample follow-up has a delivery receipt."}:{allowed:!0,label:"Ready for sample recovery",explanation:"Recorded email consent permits one sample follow-up."}:{allowed:!1,label:"Email not allowed",explanation:"No email consent was recorded. This recovery stays stopped."}}function C(e){if(!f(e)||typeof e.workspaceToken!="string")throw new Error("The sample workspace response is incomplete.");const t=e.workspace;if(!f(t)||typeof t.id!="string"||typeof t.expiresAt!="string"||!f(t.practice)||typeof t.practice.name!="string"||!f(t.service)||typeof t.service.name!="string"||!Array.isArray(t.attempts)||t.attempts.length!==3)throw new Error("The sample workspace response is incomplete.");return e}async function b(){return y("/api/v1/demo/workspaces","POST")}async function F(e){return y("/api/v1/demo/workspace","GET",e)}async function N(e){return y("/api/v1/demo/reset","POST",e)}async function B(e,t){return y(`/api/v1/demo/attempts/${encodeURIComponent(t)}/recover`,"POST",e)}async function y(e,t,o){const a=new Headers({Accept:"application/json"});t==="POST"&&a.set("Idempotency-Key",crypto.randomUUID()),o&&a.set("X-Demo-Workspace",o);let i;try{i=await fetch(e,{method:t,headers:a})}catch{throw new k("offline","The sample workspace needs a connection. Check your network and try again.",0)}const n=await i.json().catch(()=>null);if(!i.ok){const p=f(n)&&typeof n.error=="string"?n.error:"demo_unavailable",x=f(n)&&typeof n.message=="string"?n.message:"The sample workspace could not be loaded. Try again.";throw new k(p,x,i.status)}return C(n)}function f(e){return typeof e=="object"&&e!==null}const P={home:{title:"Booking Recovery Loop — recover paid sessions",description:"See where a paid booking stopped, check consent, and run a safe sample follow-up with a delivery receipt.",heading:"Recover paid sessions before they disappear",canonicalPath:"/"},demo:{title:"Demo — Booking Recovery Loop",description:"Try a consent-aware booking recovery with isolated sample data and a simulated delivery receipt.",heading:"Recover a sample booking",canonicalPath:"/demo"},privacy:{title:"Privacy — Booking Recovery Loop",description:"How Booking Recovery Loop handles temporary demo data and protects personal information.",heading:"Your sample stays separate",canonicalPath:"/privacy"},terms:{title:"Terms — Booking Recovery Loop",description:"The terms for using the Booking Recovery Loop sample workspace and future paid service.",heading:"Terms for the sample workspace",canonicalPath:"/terms"},"not-found":{title:"Page not found — Booking Recovery Loop",description:"The requested Booking Recovery Loop page was not found.",heading:"That page is not here",canonicalPath:"/404"}};function g(e,t=""){if(e==="/"&&new URLSearchParams(t).get("demo")==="1")return"demo";switch(e){case"/":return"home";case"/demo":return"demo";case"/privacy":return"privacy";case"/terms":return"terms";case"/404":return"not-found";default:return"not-found"}}function E(e,t=""){return P[g(e,t)]}function I(e){return new URL(e,"https://booking-recovery-loop.sociobot.in").toString()}const A=document.querySelector("#app");if(!A)throw new Error("The application root is missing.");const O=A;let l=null,d=!1,u=null,m=null,v=null,c=null;function H(){return`
    <section class="hero" aria-describedby="hero-summary">
      <div class="hero-copy">
        <p class="eyebrow">Booking follow-up with proof</p>
        <h1 tabindex="-1">Recover paid sessions before they disappear</h1>
        <p id="hero-summary" class="lede">For solo coaches and tutors who need to see why a paid booking stopped and what can happen next.</p>
        <div class="hero-action">
          <a class="button button-primary" href="/demo">Try it with sample data</a>
          <p>Opens a safe workspace with three fictional clients.</p>
        </div>
        <ul class="plain-facts" aria-label="Demo facts">
          <li><span aria-hidden="true">01</span> No account needed</li>
          <li><span aria-hidden="true">02</span> No real messages sent</li>
          <li><span aria-hidden="true">03</span> No payment in the demo</li>
        </ul>
      </div>
      <figure class="hero-scene">
        <img src="${M}" width="920" height="620" fetchpriority="high" alt="A calm appointment rail showing one booking that needs a follow-up." />
        <figcaption>One booking stopped. Consent decides the next step.</figcaption>
      </figure>
    </section>

    <section class="product-preview section-rule" aria-labelledby="preview-title">
      <div class="section-intro">
        <p class="eyebrow">The product</p>
        <h2 id="preview-title">See the break in the booking loop</h2>
        <p>Each ticket keeps the booking state, permission, and delivery evidence together.</p>
      </div>
      <div class="preview-board" aria-label="Sample recovery board preview">
        <div class="preview-ticket preview-ticket-muted">
          <p class="ticket-time">Tue · 14:00</p>
          <h3>Booking started</h3>
          <p>Service and time chosen</p>
          <span class="status status-good">Recorded</span>
        </div>
        <div class="preview-connector" aria-hidden="true"></div>
        <div class="preview-ticket preview-ticket-active">
          <p class="ticket-time">18 minutes ago</p>
          <h3>Deposit not finished</h3>
          <p>Email consent is on record.</p>
          <span class="status status-attention">Needs a follow-up</span>
        </div>
        <div class="preview-connector" aria-hidden="true"></div>
        <div class="preview-ticket preview-ticket-muted">
          <p class="ticket-time">Next</p>
          <h3>Delivery receipt</h3>
          <p>Waiting for a permitted action</p>
          <span class="status status-neutral">Not started</span>
        </div>
      </div>
    </section>

    <section id="how-it-works" class="how-section section-rule" aria-labelledby="how-title">
      <div class="section-intro">
        <p class="eyebrow">How it works</p>
        <h2 id="how-title">Follow one accountable path</h2>
      </div>
      <ol class="process-rail">
        <li><span>1</span><div><h3>Find the stopped booking</h3><p>See the chosen session and where the client left.</p></div></li>
        <li><span>2</span><div><h3>Check permission first</h3><p>A follow-up stays stopped when contact consent is missing.</p></div></li>
        <li><span>3</span><div><h3>Keep the receipt</h3><p>The sample action ends with a labelled delivery record.</p></div></li>
      </ol>
    </section>

    <section class="boundary-section section-rule" aria-labelledby="boundary-title">
      <div>
        <p class="eyebrow">Clear boundaries</p>
        <h2 id="boundary-title">It does not replace your calendar</h2>
      </div>
      <div class="boundary-copy">
        <p>Booking Recovery Loop focuses on the steps after someone chooses a paid session.</p>
        <p>It is not a CRM, a marketplace, or a tool for bulk messages.</p>
        <a href="/privacy">Read how the sample handles data</a>
      </div>
    </section>

    <section id="practice-plan" class="plan-section section-rule" aria-labelledby="plan-title">
      <div>
        <p class="eyebrow">Practice plan</p>
        <h2 id="plan-title">Recovery Loop Practice</h2>
        <p class="plan-price"><strong>$29</strong> / month</p>
      </div>
      <div class="plan-copy">
        <p>For one practice with one to five practitioners.</p>
        <p>The paid plan is not open in M1. Accounts and hosted checkout arrive in M2.</p>
        <a class="button button-secondary" href="/demo">Try the sample first</a>
      </div>
    </section>`}function q(){if(d&&!l)return`
      <section class="demo-heading">
        <p class="eyebrow">North Star Coaching · sample</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <div class="state-panel" role="status" aria-live="polite">
          <span class="state-marker" aria-hidden="true"></span>
          <div><h2>Preparing the sample workspace</h2><p>Adding three fictional bookings and their consent records.</p></div>
        </div>
      </section>`;if(u&&!l)return`
      <section class="demo-heading">
        <p class="eyebrow">Sample workspace</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <div class="state-panel state-panel-error" role="alert">
          <div><h2>${!navigator.onLine?"The demo is offline":"The demo did not load"}</h2><p>${s(u)}</p></div>
          <button class="button button-primary" type="button" data-action="retry-demo">Try the demo again</button>
        </div>
      </section>`;if(!l)return"";const{workspace:e}=l,t=e.attempts.find(a=>a.id===m)??e.attempts[0];if(!t)return'<section class="demo-heading"><div><p class="eyebrow">Sample workspace</p><h1 tabindex="-1">Recover a sample booking</h1><div class="state-panel state-panel-error" role="alert"><div><h2>The sample is incomplete</h2><p>Reset the demo to restore its sample bookings.</p></div><button class="button button-primary" type="button" data-action="reset-demo">Reset demo</button></div></div></section>';m=t?.id??null;const o=e.attempts.filter(a=>a.state==="unfinished").length;return`
    <section class="demo-heading">
      <div>
        <p class="eyebrow">${s(e.practice.name)} · sample</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <p class="lede">Choose a ticket, check its consent record, then run one simulated follow-up.</p>
      </div>
      <dl class="service-summary" aria-label="Sample service">
        <div><dt>Service</dt><dd>${s(e.service.name)}</dd></div>
        <div><dt>Deposit</dt><dd>${ae(e.service.depositCents,e.service.currency)}</dd></div>
        <div><dt>Needs review</dt><dd>${o}</dd></div>
      </dl>
    </section>
    ${c?`<p class="inline-notice" role="status" aria-live="polite">${s(c)}</p>`:""}
    <div class="recovery-board">
      <section class="ticket-rail" aria-labelledby="rail-title">
        <div class="rail-heading"><h2 id="rail-title">Booking rail</h2><p>Times shown for London</p></div>
        <ul class="ticket-list">
          ${e.attempts.map(a=>U(a,a.id===t.id)).join("")}
        </ul>
        <div class="empty-state" aria-label="Delivery failures">
          <span aria-hidden="true">✓</span>
          <div><h3>No missed delivery receipts</h3><p>Sample delivery problems would appear here.</p></div>
        </div>
      </section>
      <aside class="case-detail" aria-labelledby="case-title">
        ${K(t)}
      </aside>
    </div>`}function U(e,t){const o=te(e);return`<li>
    <button class="appointment-ticket ${t?"is-selected":""}" type="button" aria-pressed="${t}" data-action="select-attempt" data-attempt-id="${s(e.id)}">
      <span class="ticket-date">${oe(e.scheduledFor)}</span>
      <strong>${s(e.clientName)}</strong>
      <span>${s(e.reason)}</span>
      <span class="status ${o.className}">${o.label}</span>
    </button></li>`}function K(e){const t=L(e),o=v===e.id,a=e.consent.recordedAt?T(e.consent.recordedAt):"Not recorded",i=e.state==="completed"||e.state==="recovered"?"":`<button class="button ${t.allowed?"button-primary":"button-secondary"}" type="button" data-action="recover-attempt" data-attempt-id="${s(e.id)}" ${o?"disabled":""}>${o?"Running sample…":t.allowed?"Run sample follow-up":"Check recovery permission"}</button>`;return`
    <div class="case-topline"><span class="case-number">Selected ticket</span><span>${T(e.scheduledFor)}</span></div>
    <h2 id="case-title" tabindex="-1">${s(e.clientName)}</h2>
    <p>${s(e.reason)}.</p>
    <section class="evidence-block" aria-labelledby="consent-title">
      <div class="evidence-heading"><h3 id="consent-title">Email permission</h3><span class="status ${e.consent.email?"status-good":"status-blocked"}">${e.consent.email?"Recorded":"Missing"}</span></div>
      <p class="evidence-quote">${e.consent.wording?`“${s(e.consent.wording)}”`:"No email wording was accepted."}</p>
      <p class="evidence-time">${a}</p>
    </section>
    <section class="action-block" aria-labelledby="action-title">
      <h3 id="action-title">Next permitted step</h3>
      <p class="permission-copy ${t.allowed?"":"permission-blocked"}">${s(t.explanation)}</p>
      ${i}
      <p class="action-note">Demo actions use an in-process mailbox. No email leaves this site.</p>
    </section>
    <section class="receipt-block" aria-labelledby="receipt-title">
      <h3 id="receipt-title">Delivery evidence</h3>
      ${Z(e)}
    </section>`}function Z(e){return e.receipts.length===0?'<div class="empty-receipt"><span aria-hidden="true">○</span><p>No receipt yet. A permitted sample action will add one here.</p></div>':`<ol class="receipt-timeline">${e.receipts.map(t=>`<li>
        <span class="receipt-node" aria-hidden="true">✓</span>
        <div><strong>${ie(t.status)} · simulated ${s(t.channel)}</strong><time datetime="${s(t.occurredAt)}">${T(t.occurredAt)}</time><p>${s(t.detail)}</p></div>
      </li>`).join("")}</ol>`}function j(){return`
    <article class="policy-page">
      <p class="eyebrow">Privacy</p>
      <h1 tabindex="-1">Your sample stays separate</h1>
      <p class="policy-lede">The demo uses fictional people and a temporary workspace. It never opens a real practice record.</p>
      <section><h2>What the demo stores</h2><p>Your browser keeps one random demo token under <code>demo:workspace-token</code>.</p><p>The server keeps the matching sample workspace for up to 24 hours.</p></section>
      <section><h2>What the demo does not contact</h2><p>Demo actions do not call payment, messaging, sign-in, billing, or AI services.</p><p>The simulated receipt comes from this product’s own server.</p></section>
      <section><h2>How to remove the sample</h2><p>Reset demo deletes the current workspace and creates a fresh one.</p><p>Start for real removes the browser token. The inaccessible server copy expires automatically.</p></section>
      <section><h2>Production data</h2><p>M1 has no customer account, payment, or real contact-data flow.</p><p>This notice will change before those features open.</p></section>
      <a class="button button-primary" href="/demo">Open the sample workspace</a>
    </article>`}function G(){return`
    <article class="policy-page">
      <p class="eyebrow">Terms</p>
      <h1 tabindex="-1">Terms for the sample workspace</h1>
      <p class="policy-lede">The M1 demo is a product sample. It does not create a practice account or send a real message.</p>
      <section><h2>Use the sample safely</h2><p>Use only the fictional records already provided. Do not enter client contact details.</p></section>
      <section><h2>No payment in M1</h2><p>The sample does not take deposits or sell a subscription.</p><p>The planned practice plan will use a hosted Sociobot checkout in a later milestone.</p></section>
      <section><h2>Availability</h2><p>The sample may reset during maintenance. Use Reset demo whenever its state is unclear.</p></section>
      <section><h2>Fair use</h2><p>Automated abuse may be rate limited. A limited request returns a retry time.</p></section>
      <a class="button button-primary" href="/demo">Try the sample workspace</a>
    </article>`}function Y(){return`
    <section class="not-found-page">
      <div class="lost-ticket" aria-hidden="true"><span></span><span></span><span></span></div>
      <p class="eyebrow">404 · off the rail</p>
      <h1 tabindex="-1">That page is not here</h1>
      <p>The booking rail ends here. Return home or open the sample workspace.</p>
      <div class="button-row"><a class="button button-primary" href="/">Go to the home page</a><a class="button button-secondary" href="/demo">Try the sample</a></div>
    </section>`}function V(e){switch(e){case"home":return H();case"demo":return q();case"privacy":return j();case"terms":return G();case"not-found":return Y()}}function W(e){return[["/demo","demo","Demo"],["/#how-it-works",null,"How it works"],["/privacy","privacy","Privacy"]].map(([o,a,i])=>`<a href="${o}"${e===a?' aria-current="page"':""}>${i}</a>`).join("")}function _(){return`<aside class="demo-banner" aria-label="Demo notice">
    <p><strong>Demo</strong> — sample data, nothing is saved</p>
    <div><button type="button" data-action="reset-demo" ${d?"disabled":""}>${d?"Resetting…":"Reset demo"}</button><a href="/#practice-plan" data-action="leave-demo">Start for real</a></div>
  </aside>`}function X(e,t){const o=E(e,t),a=I(o.canonicalPath);document.title=o.title,h('meta[name="description"]',"content",o.description),h('meta[property="og:title"]',"content",o.title),h('meta[property="og:description"]',"content",o.description),h('meta[property="og:url"]',"content",a),h('meta[name="twitter:title"]',"content",o.title),h('meta[name="twitter:description"]',"content",o.description),h('link[rel="canonical"]',"href",a)}function h(e,t,o){document.querySelector(e)?.setAttribute(t,o)}function r({focusHeading:e=!1}={}){const t=g(window.location.pathname,window.location.search),o=E(window.location.pathname,window.location.search);X(window.location.pathname,window.location.search),O.innerHTML=`
    <a class="skip-link" href="#main">Skip to main content</a>
    ${t==="demo"?_():""}
    <header class="site-header">
      <a class="wordmark" href="/" aria-label="Booking Recovery Loop home"><span aria-hidden="true"></span>Booking Recovery Loop</a>
      <nav aria-label="Primary navigation">${W(t)}</nav>
    </header>
    <main id="main" class="main-${t}" tabindex="-1">${V(t)}</main>
    <footer class="site-footer">
      <p>Booking Recovery Loop keeps consent and recovery evidence on one rail.</p>
      <div><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built by Param Factory</span><span>${s("d03d83db200435a8582ea5fac676139abfb139cb")}</span></div>
      <p class="art-credit">Original rail artwork made for this product.</p>
    </footer>
    <p class="sr-only" aria-live="polite" aria-atomic="true" id="route-announcement">${s(o.heading)}</p>`,e&&document.querySelector("main h1")?.focus(),t==="demo"&&!l&&!d&&!u&&D()}async function D(e=!1){d=!0,u=null,r();try{const t=e?null:localStorage.getItem(w);let o;if(t)try{o=await F(t)}catch(a){if(!(a instanceof k)||a.status!==404)throw a;localStorage.removeItem(w),o=await b()}else o=await b();$(o)}catch(t){u=R(t)}finally{d=!1,r()}}async function z(){if(!d){d=!0,c="Restoring the original sample bookings.",r();try{const e=l?.workspaceToken??localStorage.getItem(w),t=e?await N(e):await b();$(t),m=t.workspace.attempts[0]?.id??null,c="Demo reset. The original sample bookings are ready."}catch(e){c=R(e)}finally{d=!1,r()}}}async function J(e){if(!(!l||v)){v=e,c="Checking the recorded permission.",r();try{const t=await B(l.workspaceToken,e);$(t),m=e,c="Sample follow-up delivered. A simulated receipt was added."}catch(t){c=R(t)}finally{v=null,r()}}}function $(e){l=e,localStorage.setItem(w,e.workspaceToken),m??=e.workspace.attempts[0]?.id??null}function Q(e){localStorage.removeItem(w),l=null,u=null,c=null,m=null,S(e,!0)}function S(e,t){window.history.pushState({},"",`${e.pathname}${e.search}${e.hash}`),r({focusHeading:t}),e.hash?window.requestAnimationFrame(()=>document.querySelector(e.hash)?.scrollIntoView()):window.scrollTo({top:0})}function ee(e){const t=e.target;if(!(t instanceof Element))return null;const o=t.closest("a[href]");return!o||e.defaultPrevented||e.button!==0||e.metaKey||e.ctrlKey||e.shiftKey||e.altKey?null:new URL(o.href,window.location.href).origin===window.location.origin?o:null}document.addEventListener("click",e=>{const t=e.target;if(!(t instanceof Element))return;const o=t.closest("[data-action]"),a=o?.dataset.action;if(a==="select-attempt"){m=o?.dataset.attemptId??null,r(),document.querySelector("#case-title")?.focus({preventScroll:!0});return}if(a==="reset-demo"){z();return}if(a==="retry-demo"){u=null,D();return}if(a==="recover-attempt"){const p=o?.dataset.attemptId;p&&J(p);return}const i=ee(e);if(!i)return;if(i.classList.contains("skip-link")){e.preventDefault(),document.querySelector("#main")?.focus();return}const n=new URL(i.href,window.location.href);e.preventDefault(),a==="leave-demo"?Q(n):S(n,!0)});window.addEventListener("popstate",()=>r({focusHeading:!0}));window.addEventListener("offline",()=>{g(window.location.pathname,window.location.search)==="demo"&&(c="You are offline. Viewing works, but sample actions need a connection.",r())});window.addEventListener("online",()=>{g(window.location.pathname,window.location.search)==="demo"&&(c="You are back online. Sample actions are available.",r())});function te(e){return e.state==="recovered"?{label:"Recovered in demo",className:"status-good"}:e.state==="completed"?{label:"Booking complete",className:"status-good"}:e.consent.email?{label:"Needs a follow-up",className:"status-attention"}:{label:"Stopped — no consent",className:"status-blocked"}}function oe(e){return new Intl.DateTimeFormat("en-GB",{weekday:"short",day:"numeric",month:"short",hour:"2-digit",minute:"2-digit",timeZone:"Europe/London"}).format(new Date(e))}function T(e){return new Intl.DateTimeFormat("en-GB",{day:"numeric",month:"short",hour:"2-digit",minute:"2-digit",timeZone:"Europe/London",timeZoneName:"short"}).format(new Date(e))}function ae(e,t){return new Intl.NumberFormat("en-GB",{style:"currency",currency:t}).format(e/100)}function ie(e){return`${e.charAt(0).toUpperCase()}${e.slice(1)}`}function R(e){return e instanceof Error?e.message:"The sample action failed. Try it again."}function s(e){return e.replaceAll("&","&amp;").replaceAll("<","&lt;").replaceAll(">","&gt;").replaceAll('"',"&quot;").replaceAll("'","&#039;")}r();
//# sourceMappingURL=index-BFTIsQrH.js.map
