/**
 * WebTorrent bench — magnet → metadata → first 256 KB
 * Usage: node webtorrent_bench.mjs "<magnet>" [--bytes N]
 * Prints one JSON line.
 *
 * Uses tiny-stremio-player's node_modules (same engine as the reference impl).
 */

import WebTorrent from "../../tiny-stremio-player/node_modules/webtorrent/index.js";
import http from "node:http";
import { performance } from "node:perf_hooks";
import { rm, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const DEFAULT_MAGNET =
  "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c" +
  "&dn=Big+Buck+Bunny" +
  "&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce" +
  "&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce" +
  "&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A80%2Fannounce";

const args = process.argv.slice(2);
const magnet = args[0]?.startsWith("magnet:") ? args[0] : DEFAULT_MAGNET;
const bytesIdx = args.indexOf("--bytes");
const BYTES_TARGET = bytesIdx >= 0 ? parseInt(args[bytesIdx + 1], 10) : 256 * 1024;

const cacheDir = join(tmpdir(), "fluxa_bench_webtorrent");
await rm(cacheDir, { recursive: true, force: true });
await mkdir(cacheDir, { recursive: true });

process.stderr.write("[webtorrent] starting client…\n");
const t0 = performance.now();

const client = new WebTorrent({ downloadLimit: -1, uploadLimit: 0 });

await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => {
    client.destroy();
    reject(new Error("WebTorrent timeout (120s)"));
  }, 120_000);

  client.add(magnet, { path: cacheDir }, (torrent) => {
    const tMetadata = performance.now();
    process.stderr.write(
      `[webtorrent] metadata in ${Math.round(tMetadata - t0)}ms — ` +
        `${torrent.files.length} files\n`
    );

    // pick largest video file
    const videoExts = /\.(mp4|mkv|avi|webm|m4v|mov)$/i;
    const file =
      torrent.files
        .filter((f) => videoExts.test(f.name))
        .sort((a, b) => b.length - a.length)[0] ?? torrent.files[0];

    process.stderr.write(
      `[webtorrent] target: ${file.name} (${Math.round(file.length / 1024 / 1024)} MB)\n`
    );

    // deselect all, select only the target
    torrent.files.forEach((f) => f.deselect());
    file.select();

    // tiny HTTP server (same pattern as tiny-stremio-player)
    const server = http.createServer((req, res) => {
      const size = file.length;
      const end = Math.min(BYTES_TARGET - 1, size - 1);
      res.writeHead(206, {
        "Accept-Ranges": "bytes",
        "Content-Type": "video/x-matroska",
        "Content-Range": `bytes 0-${end}/${size}`,
        "Content-Length": end + 1,
      });
      file.createReadStream({ start: 0, end }).pipe(res);
    });

    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      process.stderr.write(`[webtorrent] HTTP server on :${port}, requesting ${BYTES_TARGET / 1024}KB…\n`);

      const tReqStart = performance.now();
      let tFirstByte = null;
      let totalRead = 0;

      const req = http.request(
        { hostname: "127.0.0.1", port, path: "/", headers: { Range: `bytes=0-${BYTES_TARGET - 1}` } },
        (res) => {
          res.once("data", () => {
            tFirstByte = performance.now();
            process.stderr.write(
              `[webtorrent] first byte in ${Math.round(tFirstByte - t0)}ms (${Math.round(tFirstByte - tReqStart)}ms after request)\n`
            );
          });

          res.on("data", (chunk) => {
            totalRead += chunk.length;
          });

          res.on("end", () => {
            const tDone = performance.now();
            clearTimeout(timeout);
            server.close();
            client.destroy();

            const result = {
              engine: "webtorrent",
              engineStartupMs: 0, // in-process, no startup
              metadataMs: Math.round(tMetadata - t0),
              firstByteMs: tFirstByte != null ? Math.round(tFirstByte - t0) : null,
              readDoneMs: Math.round(tDone - tReqStart),
              bytesRead: totalRead,
              fileName: file.name,
              fileSizeBytes: file.length,
            };

            process.stderr.write(
              `[webtorrent] ${Math.round(totalRead / 1024)}KB done in ${Math.round(tDone - t0)}ms total\n`
            );
            console.log(JSON.stringify(result, null, 2));
            resolve();
          });

          res.on("error", reject);
        }
      );
      req.on("error", reject);
      req.end();
    });
  });

  client.on("error", reject);
});
