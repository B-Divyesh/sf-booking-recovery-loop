import { createServer } from "node:http";

const port = Number(process.env.INTEGRATION_FIXTURE_PORT ?? "4174");

function json(response, status, body) {
  response.writeHead(status, { "content-type": "application/json" });
  response.end(JSON.stringify(body));
}

createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", `http://127.0.0.1:${port}`);

  if (request.method === "GET" && url.pathname === "/health") {
    return json(response, 200, { status: "ok" });
  }

  if (request.method === "POST" && url.pathname === "/products/booking-recovery-loop-deposit/checkout") {
    let raw = "";
    for await (const chunk of request) raw += chunk;
    const payload = JSON.parse(raw);
    const reference = String(payload.reference ?? "missing-reference");
    return json(response, 200, {
      checkout_url: `https://checkout.dodopayments.com/session/${reference}`,
      intent_id: `fixture-${reference}`
    });
  }

  if (request.method === "GET" && url.pathname === "/products/booking-recovery-loop-deposit/verify") {
    return json(response, 200, {
      valid: url.searchParams.get("license") === "fixture-license",
      reason: url.searchParams.get("license") === "fixture-license" ? "ok" : "invalid",
      expires_at: null
    });
  }

  return json(response, 404, { error: "fixture_route_not_found" });
}).listen(port, "127.0.0.1");
