# Research: Dynamic CLI Performance Optimization

**Date:** 2026-03-21
**Agent:** general-purpose (web)

---

## Summary

A dynamically-generated CLI can achieve <30ms warm startup by using binary-cached specs with zero-copy deserialization, lazy per-product loading, and serve-stale cache strategy with background refresh.

## Performance Targets

| Scenario | Target |
|----------|--------|
| `shrug --version` | < 2ms |
| Warm cached command | < 30ms |
| Cold start with bundled spec | < 50ms |
| Cold start with network fetch | 500ms-2s |

## 1. Spec Caching Strategy

### Recommended: Binary Cache with rkyv Zero-Copy

Parse JSON once → serialize to rkyv binary format → memory-map on subsequent runs.

**Serialization benchmarks (1MB dataset):**
| Format | Serialize | Deserialize | Size |
|--------|-----------|-------------|------|
| serde_json | 4.4ms | ~5ms | 1.8MB |
| bincode | 0.64ms | ~0.8ms | 1.0MB |
| **rkyv** | 0.42ms | **0.001ms** (zero-copy) | 1.0MB |

### Cache Location & TTL
- Cache in `~/.cache/shrug/specs/`
- Store both raw JSON (for debugging) and binary cache (for speed)
- **Serve-stale strategy:** Always serve from cache immediately. After command execution, check ETag in background. Update cache for next invocation.
- Manual refresh: `shrug cache refresh`
- TTL: 24 hours (matching gws approach), but serve stale data while refreshing

### Offline Mode
- Pre-bundle known-good specs with `include_bytes!()` — instant first-run, offline resilient
- Fallback chain: binary cache → bundled spec → network fetch

## 2. Startup Time Optimization

### Lazy Per-Product Loading
Only load the spec for the requested product:
- `shrug jira issue list` → load only Jira Platform spec
- `shrug confluence page list` → load only Confluence spec
- Never load all specs at once

### Two-Phase Parsing (from gws)
1. Phase 1: Parse product name from argv[1] (no spec needed)
2. Phase 2: Load spec for that product → build CLI tree → re-parse

### Use rustls, Not native-tls
native-tls adds ~100ms cold-start on some platforms. `reqwest` with `rustls-tls` feature avoids this.

### Background Spec Refresh
After command completes, spawn background task to check for spec updates (ETag/If-Modified-Since). Updated spec available on next invocation.

## 3. Spec Size Reduction

Jira Platform spec is 2.47MB. Strategies:

### Command Manifest (stripped spec)
Strip fields not needed for command generation:
- Remove: descriptions, examples, deep response schemas, deprecated ops
- Keep: paths, methods, parameters, operationIds, tags, auth requirements
- **Estimated reduction: 40-60%** (to ~1-1.5MB)

### Pre-Process at Cache Time
On first load:
1. Parse full JSON spec
2. Extract command-relevant fields into a lean struct
3. Serialize lean struct to rkyv binary
4. Memory-map binary on subsequent runs

## 4. Command Tree Generation

### Partial Tree Building
For very large specs, only build the subtree for the requested tag/resource:
- `shrug jira issues list` → only build the "Issues" command group
- Other tag groups built lazily if user runs `shrug jira --help`

### Index File
Generate a small index mapping tag → operationIds → spec offsets. Load index first, then seek into spec only for the needed operations.

## 5. Network Performance

### Connection Pooling
Reuse `reqwest::Client` across requests (it pools connections internally). Create once, use everywhere.

### HTTP/2
`reqwest` supports HTTP/2 multiplexing. Atlassian Cloud supports HTTP/2.

### Rate Limit Awareness
- Parse `Retry-After` and `X-RateLimit-*` headers
- Exponential backoff with jitter: 2s, 4s, 8s, 16s (max 4 retries)
- `X-RateLimit-NearLimit: true` → proactively slow down batch ops
- API token traffic: burst limits only (not points-based)

### Parallel Batch Operations
For bulk operations, use configurable concurrency (default 5) with rate-limit feedback.

## 6. Memory Optimization

- Drop full spec after extracting command manifest
- Use `Arc<str>` for shared strings (operationIds, tag names)
- Memory-mapped binary cache avoids loading into heap

## 7. Benchmarking

Use `hyperfine` for CLI startup benchmarking:
```bash
hyperfine --warmup 3 'shrug jira issues list --help'
hyperfine 'shrug --version'
```

## Real-World Comparisons

| CLI | Spec Strategy | Startup |
|-----|--------------|---------|
| gws | Runtime fetch + 24h JSON cache | 15-30ms warm, 500ms-2s cold |
| stripe-cli | Build-time code gen from OpenAPI | <10ms (no runtime parsing) |
| aws-cli | Bundled JSON models, runtime load | ~200ms (Python overhead) |

shrug target: beat aws-cli, match gws warm performance, improve cold start with bundled specs.

---
*Research completed: 2026-03-21*
