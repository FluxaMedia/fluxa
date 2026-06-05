/**
 * WebTorrent HTTP server — magnet → HTTP stream URL
 * Metadata gelince "READY <url>" yazar stdout'a, sonra sonsuza kadar çalışır.
 * Usage: node webtorrent_serve.mjs "<magnet>"
 */

import WebTorrent from "../../tiny-stremio-player/node_modules/webtorrent/index.js";
import http from "node:http";
import { rm, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const MAGNET = process.argv[2] ?? (() => { throw new Error("Usage: webtorrent_serve <magnet>"); })();

const cacheDir = join(tmpdir(), "fluxa_bench_webtorrent");
await rm(cacheDir, { recursive: true, force: true });
await mkdir(cacheDir, { recursive: true });

process.stderr.write("[webtorrent] starting client…\n");

const client = new WebTorrent({ downloadLimit: -1, uploadLimit: 0 });
client.on("error", (e) => { process.stderr.write(`[webtorrent] client error: ${e}\n`); process.exit(1); });

client.add(MAGNET, { path: cacheDir }, (torrent) => {
  process.stderr.write(`[webtorrent] metadata ready — ${torrent.files.length} files\n`);

  const videoExts = /\.(mp4|mkv|avi|webm|m4v|mov)$/i;
  const file =
    torrent.files
      .filter((f) => videoExts.test(f.name))
      .sort((a, b) => b.length - a.length)[0] ?? torrent.files[0];

  // deselect all, only download target file
  torrent.files.forEach((f) => f.deselect());
  file.select();

  process.stderr.write(`[webtorrent] target: ${file.name} (${Math.round(file.length / 1024 / 1024)} MB)\n`);

  const server = http.createServer((req, res) => {
    const size = file.length;
    const rangeHeader = req.headers["range"];

    if (rangeHeader) {
      const match = rangeHeader.match(/bytes=(\d+)-(\d*)/);
      const start = parseInt(match[1], 10);
      const end = match[2] ? Math.min(parseInt(match[2], 10), size - 1) : size - 1;
      const length = end - start + 1;
      res.writeHead(206, {
        "Accept-Ranges": "bytes",
        "Content-Type": "video/x-matroska",
        "Content-Range": `bytes ${start}-${end}/${size}`,
        "Content-Length": length,
      });
      file.createReadStream({ start, end }).pipe(res);
    } else {
      res.writeHead(200, {
        "Accept-Ranges": "bytes",
        "Content-Type": "video/x-matroska",
        "Content-Length": size,
      });
      file.createReadStream().pipe(res);
    }
  });

  server.listen(0, "127.0.0.1", () => {
    const { port } = server.address();
    const url = `http://127.0.0.1:${port}/stream`;
    // stdout'a READY yaz — bench.sh bunu okur
    process.stdout.write(`READY ${url}\n`);
    process.stderr.write(`[webtorrent] listening on ${url}\n`);
  });
});
