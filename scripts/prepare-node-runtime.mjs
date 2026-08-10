import { createHash } from "node:crypto";
import { createWriteStream, mkdirSync, mkdtempSync, rmSync, copyFileSync, chmodSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { get } from "node:https";

const version = process.env.CDP_NODE_VERSION || "24.18.0";
const platform = process.env.CDP_NODE_PLATFORM || process.platform;
const arch = process.env.CDP_NODE_ARCH || process.arch;
const archivePlatform = platform === "win32" ? "win" : platform;
const suffix = platform === "win32" ? "zip" : platform === "darwin" ? "tar.gz" : "tar.xz";
const archiveName = `node-v${version}-${archivePlatform}-${arch}.${suffix}`;
const baseUrl = `https://nodejs.org/dist/v${version}`;
const temporary = mkdtempSync(join(tmpdir(), "cdp-injector-node-"));
const archivePath = join(temporary, archiveName);
const checksumsPath = join(temporary, "SHASUMS256.txt");

try {
  await download(`${baseUrl}/${archiveName}`, archivePath);
  await download(`${baseUrl}/SHASUMS256.txt`, checksumsPath);
  const checksums = await import("node:fs/promises").then(({ readFile }) => readFile(checksumsPath, "utf8"));
  const expected = checksums
    .split("\n")
    .find((line) => line.endsWith(`  ${archiveName}`))
    ?.split(/\s+/)[0];
  if (!expected) throw new Error(`Missing checksum for ${archiveName}`);
  const actual = await sha256(archivePath);
  if (actual !== expected) throw new Error(`Checksum mismatch for ${archiveName}`);

  const extracted = join(temporary, "extracted");
  mkdirSync(extracted);
  if (platform === "win32") {
    execFileSync("powershell.exe", [
      "-NoProfile",
      "-Command",
      `Expand-Archive -LiteralPath '${archivePath}' -DestinationPath '${extracted}'`,
    ]);
  } else {
    execFileSync("tar", ["-xf", archivePath, "-C", extracted]);
  }

  const root = join(extracted, basename(archiveName, `.${suffix}`));
  const source = platform === "win32" ? join(root, "node.exe") : join(root, "bin", "node");
  const destinationDir = resolve("src-tauri/resources/node");
  const destination = join(destinationDir, platform === "win32" ? "node.exe" : "node");
  rmSync(destinationDir, { recursive: true, force: true });
  mkdirSync(destinationDir, { recursive: true });
  copyFileSync(source, destination);
  if (platform !== "win32") chmodSync(destination, 0o755);
  console.log(`Prepared Node v${version} for ${platform}-${arch}: ${destination}`);
} finally {
  rmSync(temporary, { recursive: true, force: true });
}

function download(url, destination) {
  return new Promise((resolveDownload, reject) => {
    const request = get(url, (response) => {
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        response.resume();
        download(response.headers.location, destination).then(resolveDownload, reject);
        return;
      }
      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`Download failed (${response.statusCode}): ${url}`));
        return;
      }
      const output = createWriteStream(destination);
      response.pipe(output);
      output.on("finish", () => output.close(resolveDownload));
      output.on("error", reject);
    });
    request.on("error", reject);
  });
}

async function sha256(path) {
  const { createReadStream } = await import("node:fs");
  return new Promise((resolveHash, reject) => {
    const hash = createHash("sha256");
    const input = createReadStream(path);
    input.on("data", (chunk) => hash.update(chunk));
    input.on("end", () => resolveHash(hash.digest("hex")));
    input.on("error", reject);
  });
}
