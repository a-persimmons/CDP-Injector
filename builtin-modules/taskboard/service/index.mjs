import os from "node:os";

import { createTaskboardServer, resolveHost, resolvePort } from "../server/index.mjs";

const app = createTaskboardServer();
const host = resolveHost();
const address = await app.listen({ host, port: resolvePort() });
console.log(`Taskboard module listening on http://127.0.0.1:${address.port}`);

if (host === "0.0.0.0") {
  for (const entry of Object.values(os.networkInterfaces()).flat()) {
    if (entry?.family === "IPv4" && !entry.internal) {
      console.log(`Taskboard module available on LAN at http://${entry.address}:${address.port}`);
    }
  }
}

let closing = false;
let parentWatch;
async function close() {
  if (closing) return;
  closing = true;
  clearInterval(parentWatch);
  await app.close();
}

process.once("SIGINT", () => close().then(() => process.exit(0)));
process.once("SIGTERM", () => close().then(() => process.exit(0)));

const supervisorPid = process.ppid;
parentWatch = setInterval(() => {
  try {
    process.kill(supervisorPid, 0);
    return;
  } catch {
    // The injector exited; stop the module service instead of leaving its port occupied.
  }
  close().then(() => process.exit(0));
}, 1000);
parentWatch.unref();
