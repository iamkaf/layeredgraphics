import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import process from "node:process";

const root = new URL("../", import.meta.url);
const argument = process.argv[2];
const preid = process.argv[3] ?? "alpha";
const jsonFiles = [
  "package.json",
  "packages/core/package.json",
  "packages/browser/package.json",
  "packages/node/package.json",
  "apps/site/package.json",
];

if (!argument) {
  console.error("Usage: pnpm release:prepare <prerelease|patch|minor|major|VERSION> [preid]");
  process.exit(2);
}

const read = (path) => readFileSync(new URL(path, root), "utf8");
const write = (path, value) => writeFileSync(new URL(path, root), value);
const parseVersion = (version) => {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+)\.(\d+))?$/.exec(version);
  if (!match) throw new Error(`Unsupported semantic version: ${version}`);
  return { major: Number(match[1]), minor: Number(match[2]), patch: Number(match[3]), preid: match[4], pre: match[5] === undefined ? undefined : Number(match[5]) };
};
const formatVersion = ({ major, minor, patch, preid: id, pre }) => `${major}.${minor}.${patch}${id === undefined ? "" : `-${id}.${pre}`}`;

const rootPackage = JSON.parse(read("package.json"));
const current = rootPackage.version;
for (const path of jsonFiles) {
  const version = JSON.parse(read(path)).version;
  if (version !== current) throw new Error(`${path} has version ${version}; expected ${current}`);
}
const cargoVersion = /\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/.exec(read("Cargo.toml"))?.[1];
if (cargoVersion !== current) throw new Error(`Cargo.toml has version ${cargoVersion}; expected ${current}`);
if (argument === "check") {
  console.log(`All release manifests use ${current}`);
  process.exit(0);
}

const parsed = parseVersion(current);
let next;
if (/^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+\.\d+)?$/.test(argument)) {
  next = argument;
} else if (argument === "prerelease") {
  next = formatVersion({
    major: parsed.major,
    minor: parsed.minor,
    patch: parsed.patch,
    preid,
    pre: parsed.preid === preid && parsed.pre !== undefined ? parsed.pre + 1 : 0,
  });
} else if (argument === "patch") {
  next = formatVersion(parsed.preid === undefined
    ? { major: parsed.major, minor: parsed.minor, patch: parsed.patch + 1 }
    : { major: parsed.major, minor: parsed.minor, patch: parsed.patch });
} else if (argument === "minor") {
  next = formatVersion({ major: parsed.major, minor: parsed.minor + 1, patch: 0 });
} else if (argument === "major") {
  next = formatVersion({ major: parsed.major + 1, minor: 0, patch: 0 });
} else {
  throw new Error(`Unknown release bump: ${argument}`);
}

if (next === current) throw new Error(`Version is already ${current}`);

for (const path of jsonFiles) {
  const contents = read(path);
  const updated = contents.replace(/("version"\s*:\s*")[^"]+/, `$1${next}`);
  if (contents === updated) throw new Error(`Could not update version in ${path}`);
  write(path, updated);
}

let cargo = read("Cargo.toml");
cargo = cargo.replace(/(\[workspace\.package\][\s\S]*?\nversion = ")[^"]+("\n)/, `$1${next}$2`);
write("Cargo.toml", cargo);

for (const path of ["crates/lg-cli/Cargo.toml", "crates/lg-node/Cargo.toml", "crates/lg-wasm/Cargo.toml"]) {
  const contents = read(path);
  const updated = contents.replace(`version = "${current}", path = "../lg-core"`, `version = "${next}", path = "../lg-core"`);
  if (contents === updated) throw new Error(`Could not update internal dependency in ${path}`);
  write(path, updated);
}

const date = new Date().toISOString().slice(0, 10);
let changelog = read("CHANGELOG.md");
if (!changelog.includes(`## ${next} - `)) changelog = changelog.replace("## Unreleased\n", `## Unreleased\n\n## ${next} - ${date}\n`);
write("CHANGELOG.md", changelog);

execFileSync("cargo", ["check", "--workspace"], { cwd: root, stdio: "inherit" });
execFileSync("pnpm", ["install", "--lockfile-only", "--no-frozen-lockfile"], { cwd: root, stdio: "inherit" });

if (process.env.GITHUB_OUTPUT) writeFileSync(process.env.GITHUB_OUTPUT, `version=${next}\ntag=v${next}\n`, { flag: "a" });
console.log(`Prepared ${current} -> ${next}`);
