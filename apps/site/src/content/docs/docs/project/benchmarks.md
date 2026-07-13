---
title: Benchmarks
description: Reproducible sprite, 2K, 4K, deep-document and batch baselines.
---

The release corpus records medians, p95, throughput, peak resident memory and cache behavior. It covers document open/save, shallow/deep transactions, sprite/2K/4K reference output, retained previews and a 32-output shared-image batch.

```bash
pnpm benchmark
```

On the checked Ryzen 9 7900X Linux release run, a 40-layer shared-image sprite frame had a 14.1 ms median, retained 4K composition had a 1.21 s median, and warm multi-size batch items improved from 55.3 ms cold to 46.5 ms. These are guardrails for this host, not universal product claims.

See the [methodology, full table and interpretation](https://github.com/iamkaf/layeredgraphics/blob/main/docs/BENCHMARKS.md).
