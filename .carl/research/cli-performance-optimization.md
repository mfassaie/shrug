# CLI Performance Optimization: Dynamic OpenAPI Spec-Driven Command Generation

## Research Date: 2026-03-21

---

## 1. OpenAPI Spec Caching Strategies

### Cache Location and Format

| Strategy | Description | Estimated Impact |
|---|---|---|
| **Raw JSON on disk** | Store specs as-is in `~/.config/shrug/cache/` | Baseline; simple but requires re-parsing on every load |
| **Pre-processed binary (bincode/rkyv)** | Parse JSON once, serialize to binary format | 5-10x faster deserialization (see benchmarks below) |
| **Dual-layer cache** | Keep both raw JSON (for debugging) and binary (for speed) | Best of both worlds at cost of ~2x disk usage |

**Recommendation:** Use `~/.config/shrug/cache/{product}/{version}/` directory structure (similar to AWS CLI's botocore model). Store both the original JSON (for cache validation via ETag) and a pre-processed binary format for fast loading.

### TTL Strategies

| Strategy | Pros | Cons |
|---|---|---|
| **Fixed 24-hour TTL** (gws approach) | Simple, predictable | Misses updates for 24h; wastes bandwidth if spec unchanged |
| **ETag/If-Modified-Since** | Only downloads when changed; saves bandwidth | Requires one HTTP round-trip to check |
| **Background refresh** | Zero-latency for user; always fresh | Complexity; stale data possible |
| **Hybrid: serve from cache + background check** | Best UX: instant startup, eventual freshness | Most complex to implement |

**Recommendation:** Hybrid approach:
1. Always serve from cache if available (instant startup)
2. After command execution, check ETag in background
3. If spec changed, download and update cache for next invocation
4. Respect `Cache-Control` and `ETag` headers from Atlassian's API
5. Fall back to fixed 24-hour TTL if server doesn't support conditional requests

### Conditional Fetches (ETag / If-Modified-Since)

```
# First fetch: store the ETag
GET /rest/api/3/openapi.json
-> ETag: "abc123"
-> Store spec + ETag in cache

# Subsequent check:
GET /rest/api/3/openapi.json
If-None-Match: "abc123"
-> 304 Not Modified (no body transferred, ~50ms)
-> OR 200 OK with new spec
```

When both `If-None-Match` and `If-Modified-Since` are present, `If-None-Match` takes precedence per RFC 7232. Use ETag as the primary mechanism.

### Offline Mode

Inspired by Cargo's `--offline` flag:
1. **Automatic detection:** If network request fails, fall back to cached spec (even if expired)
2. **Explicit flag:** `shrug --offline` skips all network checks
3. **Graceful messaging:** "Using cached Jira spec from 2 days ago (offline mode)"
4. **Config option:** `net.offline = true` in config file for persistent offline mode

### Pre-bundled/Fallback Specs

- Ship a known-good version of each Atlassian product's OpenAPI spec embedded in the binary
- Use `include_bytes!()` at compile time for zero-cost embedding
- These serve as fallback when:
  - First run with no network
  - Cache is corrupted
  - Atlassian's spec endpoint is down
- Version the bundled specs and log when they're being used as fallback

### Spec Versioning and Update Detection

- Store spec metadata: `{ etag, last_modified, fetched_at, spec_version, hash }`
- Compare SHA-256 hash of new spec vs cached to detect actual content changes
- Log when spec changes are detected for debugging

### Cache Invalidation

- **Manual:** `shrug cache clear` or `shrug --refresh-cache`
- **Automatic:** TTL expiry + background ETag check
- **Per-product:** `shrug cache clear jira` to refresh only one product's spec
- **Atomic writes:** Write new cache to temp file, then rename (prevents corruption)

---

## 2. Startup Time Optimization

### Target: < 100ms for cached commands

Baseline Rust binary startup: ~0.5ms. Budget breakdown:
- Binary startup + argument parsing: ~1-2ms
- Cache check (file stat): ~1ms
- Binary deserialization of pre-processed spec: ~5-15ms
- Command tree construction: ~5-10ms
- **Total target: ~15-30ms for cached warm start**

### The Cold Start Problem

First run requires fetching specs. Atlassian products:
- Jira Platform: ~2.5MB spec
- Jira Software: ~500KB spec
- Confluence: ~1.5MB spec
- Bitbucket: ~1MB spec
- Service Management: ~800KB spec

**Mitigation strategies:**
1. **Only fetch the requested product's spec** (lazy loading by product)
2. **Show progress indicator** during first fetch
3. **Pre-bundled fallback specs** enable instant first use
4. **Parallel fetching** if multiple products needed

### Lazy Loading: Only Load What's Needed

Inspired by gws's two-phase parsing:

**Phase 1 (static, ~1ms):**
- Parse `argv[1]` to identify the product (jira, confluence, etc.)
- No spec loading needed yet

**Phase 2 (dynamic, ~10-30ms cached):**
- Load only that product's pre-processed spec from cache
- Build only the relevant command subtree
- Parse remaining arguments against the dynamic tree

This means `shrug jira issue list` never loads the Confluence spec.

### Pre-processed/Compiled Spec Formats

**Serialization benchmark comparison (from rust_serialization_benchmark):**

| Format | Serialize | Deserialize | Size | Notes |
|---|---|---|---|---|
| **serde_json** | 4.4ms | ~5ms | 1.8MB | Baseline; human-readable |
| **bincode** | 0.64ms | ~0.8ms | 1.0MB | 6-7x faster than JSON |
| **rkyv** | 0.42ms | **0.001ms*** | 1.0MB | Zero-copy deserialization |
| **simd-json** | - | ~1.5ms | same as JSON | 3x faster JSON parsing |

*rkyv's zero-copy deserialization accesses data in-place via memory mapping, achieving ~1ns "deserialization" — the data is simply cast to the target type.

**Recommendation:** Use **rkyv** for the pre-processed cache format:
- First load: Parse JSON with serde_json, serialize to rkyv, write to cache
- Subsequent loads: Memory-map the rkyv file (~1ns "deserialization")
- Fallback: If rkyv cache is invalid, re-parse from JSON cache

### JSON Parser Selection

If parsing raw JSON is needed (first load, cache rebuild):

| Parser | Performance vs serde_json | Ecosystem | Safety |
|---|---|---|---|
| **serde_json** | Baseline | 26,916 dependents, gold standard | Safe |
| **simd-json** | ~3x faster on large docs | 66 dependents, lots of unsafe | Has unsafe code |
| **sonic-rs** | ~2-4x faster | Newer, SIMD-based | Has unsafe code |

**Recommendation:** Use serde_json for correctness and ecosystem compatibility. The JSON parsing only happens on cache miss (first load or spec update), so the 3x speedup of simd-json doesn't justify the unsafe code for this use case. The pre-processed binary cache eliminates JSON parsing from the hot path entirely.

### Background Spec Refresh

```
User runs: shrug jira issue list
1. Load cached spec (instant)
2. Execute command
3. After response is displayed, spawn background task:
   - Check ETag against Atlassian's server
   - If changed, download new spec
   - Pre-process to binary format
   - Atomically update cache
4. Next invocation uses fresh spec
```

### Parallel Spec Fetching

For commands that span products (e.g., cross-product search):
- Use `tokio::join!` or `futures::join_all` to fetch multiple specs concurrently
- reqwest Client automatically handles connection pooling

---

## 3. Spec Size Reduction

### Atlassian Spec Sizes (approximate)

| Product | Raw JSON | Gzipped | Endpoints |
|---|---|---|---|
| Jira Platform | ~2.5MB | ~300KB | 300+ |
| Confluence | ~1.5MB | ~180KB | 200+ |
| Bitbucket | ~1MB | ~120KB | 150+ |
| JSM | ~800KB | ~100KB | 100+ |

### Stripping Unnecessary Fields

For command generation, you only need:
- Paths and HTTP methods
- Operation IDs and summaries (for help text)
- Parameter definitions (names, types, required/optional)
- Request body schema (top-level fields only)
- Response status codes

You do NOT need:
- Detailed descriptions (save ~30-40% of spec size)
- Example values
- Deeply nested response schemas
- External documentation links
- Server variables and security scheme details beyond what's needed

**Estimated reduction: 40-60% of spec size** by creating a minimal "command manifest."

### Command Manifest Format

Create a stripped-down intermediate format:

```rust
struct CommandManifest {
    product: String,
    version: String,
    spec_hash: String,  // for cache invalidation
    resources: Vec<Resource>,
}

struct Resource {
    name: String,
    path: String,
    methods: Vec<Method>,
}

struct Method {
    http_method: HttpMethod,
    operation_id: String,
    summary: String,  // short help text
    parameters: Vec<Parameter>,
    body_fields: Vec<BodyField>,
}
```

This manifest is what gets serialized to rkyv for the binary cache.

### Compression

- Store raw JSON cache with gzip compression (2.5MB -> ~300KB on disk)
- Binary cache (rkyv/bincode) is already compact; compression optional
- For network transfers, request `Accept-Encoding: gzip` (most servers support this)

---

## 4. Command Tree Generation Performance

### Pre-computed vs Runtime Generation

| Approach | Startup Time | Freshness | Complexity |
|---|---|---|---|
| **Stripe CLI: build-time codegen** | ~0ms (compiled in) | Requires new release | Low runtime complexity |
| **AWS CLI: bundled JSON models** | ~50-200ms (parse on start) | Requires new release | Medium |
| **gws: runtime from Discovery** | ~15-30ms (cached) | Always fresh | High |
| **Recommended: cached binary manifest** | ~5-15ms | Fresh within TTL | Medium |

### Serialized Command Trees

Instead of rebuilding the clap::Command tree from the spec every time:

1. Build the command tree once from the spec
2. Serialize the command metadata (not the clap objects themselves) to a manifest
3. On subsequent runs, load the manifest and construct clap::Command objects directly

This avoids the overhead of spec interpretation on every invocation.

### Partial Tree Building

Only build the subtree the user actually needs:

```
shrug jira issue create
       ^    ^     ^
       |    |     +-- Only need methods for "issue" resource
       |    +-------- Only need "issue" resource from jira spec
       +------------- Only load jira spec
```

Three levels of lazy loading:
1. **Product level:** Only load the requested product's spec
2. **Resource level:** Only parse the requested resource's methods
3. **Method level:** Only parse parameters for the matched method

### Index-Based Lookup

Create an index file per product:
```json
{
  "jira": {
    "resources": {
      "issue": { "offset": 1024, "length": 8192 },
      "project": { "offset": 9216, "length": 4096 }
    }
  }
}
```

Use this to seek directly to the relevant section of the spec. Combined with rkyv's zero-copy access, this enables loading only the bytes needed.

---

## 5. Network Performance

### Connection Pooling (reqwest)

reqwest provides built-in connection pooling via its `Client` struct:
- **Create one Client and reuse it** (it uses Arc internally)
- Default: persistent connections with keep-alive
- Configure: `pool_idle_timeout`, `pool_max_idle_per_host`
- Client already handles HTTP/2 multiplexing when supported

```rust
// Create once, reuse everywhere
let client = reqwest::Client::builder()
    .pool_idle_timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(5)
    .tcp_keepalive(Duration::from_secs(60))
    .tcp_nodelay(true)
    .build()?;
```

### HTTP/2 Multiplexing

reqwest supports HTTP/2 automatically when the server supports it. This is beneficial for:
- Parallel API calls in batch operations
- Concurrent spec fetching for multiple products

### Timeout and Retry Strategy

```rust
let client = reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(5))
    .timeout(Duration::from_secs(30))
    .build()?;

// Retry with exponential backoff
// 1s -> 2s -> 4s, max 3 retries
```

### Atlassian Rate Limiting

As of March 2026, Atlassian enforces a **points-based rate limiting model**:
- Each API call consumes points based on complexity and data returned
- Three independent systems: quota limits (hourly), burst limits (per-second), concurrent limits
- **API token-based traffic** has separate burst limits (not affected by new points model)
- Rate limit headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `Retry-After`

**Shrug should:**
- Parse rate limit response headers
- Implement exponential backoff on 429 responses
- Respect `Retry-After` header
- Show user-friendly messages: "Rate limited. Retrying in 5s..."
- For batch operations, pace requests to stay under limits
- Track point consumption across commands in a session

### Parallel Requests for Batch Operations

```rust
// Fetch multiple resources concurrently
let results = futures::future::join_all(
    urls.iter().map(|url| client.get(url).send())
).await;
```

Use a semaphore to limit concurrency:
```rust
let semaphore = Arc::new(Semaphore::new(10)); // max 10 concurrent requests
```

---

## 6. Memory Optimization

### Streaming Parse vs Full Load

For a 2.5MB Jira spec:

| Approach | Peak Memory | Parse Time | Complexity |
|---|---|---|---|
| **Full load (serde_json::from_str)** | ~10-15MB | ~5ms | Low |
| **Streaming (serde StreamDeserializer)** | ~2-3MB | ~7ms | High |
| **Memory-mapped rkyv** | ~2.5MB (mapped) | ~0.001ms | Medium |

**Recommendation:** Memory-mapped rkyv is the clear winner. The spec is mapped directly from disk into the process address space, with zero deserialization cost. The OS handles paging, so only accessed portions consume physical RAM.

Rust libraries for streaming JSON if needed:
- `serde_json::StreamDeserializer` for sequential top-level values
- `struson` for true streaming/SAX-style parsing
- `json_stream` for processing files larger than available RAM

### Reference-Counted Schemas

OpenAPI specs often reuse schemas via `$ref`. When building the command tree:
- Resolve `$ref` references at parse time
- Use `Arc<Schema>` for shared schema definitions
- A typical spec may have 50-100 reusable schemas

### Dropping Unused Data

After building the command manifest:
```rust
// Parse full spec
let spec: OpenApiSpec = parse_spec(&json_data);

// Build minimal command manifest
let manifest: CommandManifest = build_manifest(&spec);

// Drop the full spec - reclaims ~10-15MB
drop(spec);

// Serialize manifest to binary cache
write_binary_cache(&manifest)?;
```

---

## 7. Benchmarking and Monitoring

### CLI Startup Time Targets

| Category | Target | Description |
|---|---|---|
| **Instant** | < 50ms | User perceives no delay |
| **Fast** | 50-100ms | Slight delay but feels responsive |
| **Acceptable** | 100-200ms | Noticeable but tolerable |
| **Slow** | 200-500ms | Users notice and get frustrated |
| **Unacceptable** | > 500ms | Feels broken |

Reference points:
- Rust binary startup: ~0.5ms
- ripgrep startup: ~3ms
- gws cached command: ~15-30ms
- gws cold start (network fetch): 500ms-2s

### Benchmarking with Hyperfine

```bash
# Measure warm start (cached spec)
hyperfine --warmup 3 'shrug jira issue list --help'

# Compare with cold start
hyperfine --prepare 'rm -rf ~/.config/shrug/cache' 'shrug jira issue list --help'

# Compare binary cache formats
hyperfine \
  'shrug --cache-format json jira issue list --help' \
  'shrug --cache-format bincode jira issue list --help' \
  'shrug --cache-format rkyv jira issue list --help'

# Use --shell=none for sub-5ms commands
hyperfine -N 'shrug --version'
```

### Internal Timing Instrumentation

```rust
// Add --timing flag for development
if args.timing {
    eprintln!("spec_load:    {:>6.2}ms", spec_load_time.as_millis());
    eprintln!("tree_build:   {:>6.2}ms", tree_build_time.as_millis());
    eprintln!("arg_parse:    {:>6.2}ms", arg_parse_time.as_millis());
    eprintln!("api_call:     {:>6.2}ms", api_call_time.as_millis());
    eprintln!("total:        {:>6.2}ms", total_time.as_millis());
}
```

### TLS Backend Choice

**Critical finding:** Using `native-tls` with reqwest can add ~100ms to cold start on some platforms. Using `rustls` eliminates this overhead.

```toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
```

---

## 8. Real-World Architecture Comparisons

### Google Workspace CLI (gws)

| Aspect | Implementation |
|---|---|
| **Language** | Rust |
| **Spec source** | Google Discovery Service (JSON) |
| **Cache location** | `~/.config/gws/discovery_cache/` |
| **Cache format** | Raw JSON on disk |
| **Cache TTL** | 24 hours |
| **Command framework** | clap (dynamic Command construction) |
| **Architecture** | Two-phase parsing: identify service -> fetch spec -> build tree -> re-parse args |
| **Cold start** | 500ms-2s (network fetch required) |
| **Warm start** | ~15-30ms (JSON parse from disk cache) |
| **Offline mode** | None (fails if no cache and no network) |
| **Pre-bundled specs** | None |

**Lessons for shrug:**
- gws's two-phase parsing is the right architecture
- gws leaves performance on the table by not using binary serialization (re-parses JSON every run)
- gws has no offline fallback — we should do better
- 24-hour TTL is reasonable but hybrid ETag approach is better

### Stripe CLI

| Aspect | Implementation |
|---|---|
| **Language** | Go |
| **Spec source** | OpenAPI 3.x specs (3 files: v1, v2, v2-preview) |
| **Architecture** | **Build-time code generation** — specs are processed at compile time |
| **Command framework** | Cobra (Go) |
| **Key difference** | No runtime spec parsing at all; commands are compiled Go code |
| **Startup** | Very fast (no spec parsing needed) |
| **Freshness** | Requires new CLI release for API updates |

**Lessons for shrug:**
- Build-time codegen is fastest but sacrifices dynamic updates
- Good hybrid: ship pre-bundled specs for instant first start, update dynamically at runtime
- Stripe's template-based generation (`resources_cmds.go.tpl`) is a clean architecture

### AWS CLI (botocore)

| Aspect | Implementation |
|---|---|
| **Language** | Python |
| **Spec source** | Service model JSON files bundled in botocore package |
| **Model location** | `botocore/data/{service}/{version}/service-2.json` |
| **Architecture** | Models shipped with package, loaded at runtime |
| **Custom models** | Users can drop models in `~/.aws/models/` |
| **Command generation** | Fully dynamic from JSON models — no client code written |
| **Versioning** | Version in filename; no JSON parsing needed to determine version |

**Lessons for shrug:**
- AWS's directory structure (`{service}/{version}/service-2.json`) is clean and extensible
- Bundling models with the binary is the most reliable approach
- Supporting user-provided models (`~/.shrug/models/`) is valuable for customization
- Version in filename is a smart optimization to avoid parsing

---

## Summary of Recommendations (Priority-Ordered)

### Must-Have (P0) — Estimated Total Impact: Startup < 30ms

1. **Two-phase parsing architecture** (like gws): Only load the spec for the requested product
2. **Binary cache format** (rkyv): ~1ns deserialization vs ~5ms for JSON
3. **24-hour TTL with ETag validation**: Balances freshness and speed
4. **rustls instead of native-tls**: Avoid ~100ms cold-start penalty
5. **Pre-bundled fallback specs**: Instant first-run experience

### Should-Have (P1) — Estimated Impact: Better UX and Resilience

6. **Background spec refresh**: Check for updates after command executes
7. **Command manifest** (stripped spec): Reduce parse/load size by 40-60%
8. **Offline mode** with graceful fallback to cached/bundled specs
9. **Partial tree building**: Only build the subtree for the requested resource
10. **Internal timing instrumentation**: `--timing` flag for performance debugging

### Nice-to-Have (P2) — Estimated Impact: Marginal Gains

11. **Index-based lookup** for sub-resource spec loading
12. **Delta updates** instead of full spec re-download
13. **Memory-mapped binary cache** for zero-copy spec access
14. **Parallel spec fetching** for multi-product commands
15. **Compression** for on-disk cache (gzip)

### Architecture Decision: Cache Pipeline

```
Atlassian API
     |
     v
[Raw JSON spec] -- gzip compressed on disk (~300KB)
     |
     v
[Command Manifest] -- stripped to essentials (~40% of original)
     |
     v
[rkyv binary cache] -- zero-copy deserialization (~1ns load)
     |
     v
[clap::Command tree] -- built from manifest in ~5-10ms
```

### Expected Performance Profile

| Scenario | Time | Notes |
|---|---|---|
| `shrug --version` | < 2ms | No spec loading |
| `shrug jira issue list` (warm) | < 30ms | Binary cache hit |
| `shrug jira issue list` (cold, bundled) | < 50ms | Using pre-bundled spec |
| `shrug jira issue list` (cold, network) | 500ms-2s | First-ever run, no bundled spec |
| Tab completion | < 50ms | Partial tree from binary cache |
