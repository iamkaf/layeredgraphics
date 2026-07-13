# Benchmark methodology and checked results

Run the release corpus with:

```bash
./scripts/run-benchmarks.sh
pnpm benchmark
```

The runner covers open/save, shallow/deep command execution, sprite/2K/4K reference rendering, retained sessions, and a 32-output shared-image batch. It records sample count, median, p95, throughput, peak resident memory and retained-cache behavior. `pnpm benchmark` writes a temporary result and applies `benchmarks/budgets.json`.

The checked baselines are `benchmarks/results-linux-ryzen7900x.json` and `benchmarks/results-browser-chromium149.json`: release build, AMD Ryzen 9 7900X (12 cores/24 threads), x86-64 Linux. The browser lifecycle suite used headless Chromium 149.0.7827.55 on the same host; WebGPU was unavailable in that headless run, so it exercised the Canvas2D fallback (interactive p95 1.3 ms; preview p95 2.0 ms across 12 warm samples each). Dedicated WebGPU composition/loss is shader-validated and covered with a deterministic device recreation test.

Selected medians from the checked run:

| Workload | Median | p95 | Cache/result |
| --- | ---: | ---: | --- |
| Open 1 MiB embedded `.kgfx` | 0.54 ms | 4.32 ms | 15 samples |
| CLI startup/version | 0.90 ms | 1.87 ms | 20 processes |
| Reference sprite 128 | 0.98 ms | 3.35 ms | 25 samples |
| Reference 2K | 322 ms | 329 ms | 5 samples |
| Reference 4K | 1,330 ms | 1,330 ms | 2 samples |
| Retained sprite, 40 shared-image layers | 14.1 ms | 19.6 ms | 1,200 hits / 40 misses |
| Sprite visibility mutation | 12.8 ms | 18.2 ms | 30 revisions |
| Retained 2K, 6 layers | 578 ms | 590 ms | 100.2 MiB cache |
| Retained 4K, 3 layers | 1,214 ms | 1,328 ms | 201.1 MiB cache |
| Deep retained document | 1.91 ms | 7.29 ms | 241 cached group/source entries |
| Deep top opacity mutation | 1.99 ms | 7.89 ms | 12 revisions |
| Deep bottom source mutation | 4.74 ms | 9.86 ms | 3 conservative rebuilds |
| Cold batch item | 55.3 ms | 61.0 ms | 32 outputs, 3 sizes |
| Warm batch item | 46.5 ms | 49.8 ms | 1.19× median speedup |

Peak resident memory for the complete sequential run was 273.7 MiB. The retained cache itself remained below its 256 MiB budget. These numbers are initial guardrails, not broad hardware claims. Correctness failures are hard gates; noisy timing regressions require explicit review and a new checked result if accepted.

The browser smoke workflow additionally asserts that a worker preview allows a main-thread animation frame to run, refined retained pixels exactly match a cold render, transforms carry both dirty bounds, stale queued work is coalesced, fallback presentation works, device-loss presentation recovers, and batch outputs remain ordered and chunk-bounded.
