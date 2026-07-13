import fs from "node:fs";

const resultPaths = process.argv.slice(2);
if (!resultPaths.length) resultPaths.push("benchmarks/results-linux-ryzen7900x.json", "benchmarks/results-browser-chromium149.json");
const resultSets = resultPaths.map((path) => JSON.parse(fs.readFileSync(path, "utf8")));
const results = resultSets[0];
const config = JSON.parse(fs.readFileSync("benchmarks/budgets.json", "utf8"));
const timingTolerance = Number(process.env.LG_BENCHMARK_TIMING_TOLERANCE ?? 1);
if (!Number.isFinite(timingTolerance) || timingTolerance < 1) throw new Error("LG_BENCHMARK_TIMING_TOLERANCE must be a number >= 1");
const measurements = new Map(resultSets.flatMap((set) => set.measurements).map((item) => [item.workload, item]));
const failures = [];

for (const [name, budget] of Object.entries(config.budgets)) {
  const result = measurements.get(name);
  if (!result) continue;
  const p95Limit = budget.p95Ms === undefined ? undefined : budget.p95Ms * timingTolerance;
  if (p95Limit !== undefined && result.p95Ms > p95Limit) failures.push(`${name} p95 ${result.p95Ms.toFixed(2)}ms > ${p95Limit.toFixed(2)}ms`);
  if (budget.maximumCacheBytes !== undefined && result.cacheBytes > budget.maximumCacheBytes) failures.push(`${name} cache ${result.cacheBytes} > ${budget.maximumCacheBytes}`);
  if (budget.mainThreadLongTasks !== undefined && result.mainThreadLongTasks > budget.mainThreadLongTasks) failures.push(`${name} main-thread long tasks ${result.mainThreadLongTasks} > ${budget.mainThreadLongTasks}`);
  if (budget.maximumPendingPreviews !== undefined && result.maximumPendingPreviews > budget.maximumPendingPreviews) failures.push(`${name} pending previews ${result.maximumPendingPreviews} > ${budget.maximumPendingPreviews}`);
  if (budget.maximumConcurrency !== undefined && result.maximumConcurrency > budget.maximumConcurrency) failures.push(`${name} concurrency ${result.maximumConcurrency} > ${budget.maximumConcurrency}`);
  if (budget.maximumRetainedBytes !== undefined && result.maximumRetainedBytes > budget.maximumRetainedBytes) failures.push(`${name} retained bytes ${result.maximumRetainedBytes} > ${budget.maximumRetainedBytes}`);
  if (budget.minimumWarmCacheHitRate !== undefined) {
    const rate = result.cacheHits / (result.cacheHits + result.cacheMisses);
    if (rate < budget.minimumWarmCacheHitRate) failures.push(`${name} cache hit rate ${rate.toFixed(3)} < ${budget.minimumWarmCacheHitRate}`);
  }
  if (budget.minimumColdToWarmSpeedup !== undefined) {
    const cold = measurements.get(name.replace("warm", "cold"));
    const speedup = cold?.medianMs / result.medianMs;
    if (!speedup || speedup < budget.minimumColdToWarmSpeedup) failures.push(`${name} speedup ${speedup?.toFixed(3)} < ${budget.minimumColdToWarmSpeedup}`);
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log(`benchmark budgets passed (${resultSets.map((set) => set.host.browser ?? set.host.cpu ?? set.host.arch).join("; ")})`);
