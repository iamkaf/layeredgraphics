import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, normalize } from "node:path";

const root = new URL("../dist/", import.meta.url).pathname;
const types = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".kgfx": "application/vnd.layered-graphics",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
  ".xml": "application/xml",
};

createServer(async (request, response) => {
  try {
    const pathname = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    const relative = normalize(pathname).replace(/^[/\\]+/, "");
    let file = join(root, relative);
    if (!file.startsWith(root)) throw new Error("unsafe path");
    const info = await stat(file).catch(() => undefined);
    if (!info || info.isDirectory()) file = join(file, "index.html");
    const finalInfo = await stat(file);
    if (!finalInfo.isFile()) throw new Error("not a file");
    response.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" });
    createReadStream(file).pipe(response);
  } catch {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end("Not found");
  }
}).listen(4327, "127.0.0.1", () => {
  console.log("Serving dist at http://127.0.0.1:4327");
});
