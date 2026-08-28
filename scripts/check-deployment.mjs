import { readFile } from "node:fs/promises";

const path = new URL("../deploy/containerapp.m1.json", import.meta.url);
const config = JSON.parse(await readFile(path, "utf8"));

if (config.artifactClass !== "web-with-backend") {
  throw new Error("The deployment must remain web-with-backend.");
}
if (config.containerPort !== 8080) {
  throw new Error("The deployment must expose port 8080.");
}
if (config.scale?.minReplicas !== 1 || config.scale?.maxReplicas !== 1) {
  throw new Error(
    "M1 must stay on one replica so its local per-client limiter is service-wide.",
  );
}

console.log("M1 deployment boundary: one ingress-routed replica on port 8080");
