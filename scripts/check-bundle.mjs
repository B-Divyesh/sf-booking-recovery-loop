import { gzipSync } from "node:zlib";
import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";

const limits = { ".js": 200 * 1024, ".css": 50 * 1024 };
const assets = "dist/assets";
let failed = false;

for (const name of await readdir(assets)) {
  const extension = Object.keys(limits).find((item) => name.endsWith(item));
  if (!extension) continue;
  const path = join(assets, name);
  const bytes = extension === ".js" ? gzipSync(await readFile(path)).byteLength : (await stat(path)).size;
  const limit = limits[extension];
  console.log(`${name}: ${bytes} bytes (${extension === ".js" ? "gzip" : "raw"}; limit ${limit})`);
  if (bytes > limit) failed = true;
}

if (failed) process.exitCode = 1;
