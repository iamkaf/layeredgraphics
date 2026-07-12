import { cp, mkdir } from "node:fs/promises";

await mkdir(new URL("../dist/generated/", import.meta.url), { recursive: true });
await cp(new URL("../src/generated/", import.meta.url), new URL("../dist/generated/", import.meta.url), {
  recursive: true,
});
