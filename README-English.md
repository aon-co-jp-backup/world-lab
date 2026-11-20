# World Laboratory (world-lab)

**This project is at the concept stage.** The goal is a distributed
computing foundation — BOINC/Folding@home-style volunteer computing —
that lets people around the world contribute the spare compute capacity
of their PCs, tablets, and phones (GPU/NPU/CPU) to AI workloads (LLM
inference, training, data processing, etc.), making research-lab /
university / vendor-scale AI development environments accessible to
individuals who could never afford them alone.

## Why it was split off

This started as a sub-concept of
[`dream-os`](https://github.com/aon-co-jp/dream-os) (the Linux×TRON
hybrid kernel concept), but "execution backend" (dispatching kernels on
GPU/NPU) and "distributed coordination" (tying together nodes across the
world) are different layers with different concerns, so it was split
into its own repository (2026-08-06).

## Place within the ecosystem

- [`dream-os`](https://github.com/aon-co-jp/dream-os) — the lower-layer
  execution backend (shared Windows/Android Vulkan backend, mining
  kernel, Toshiba-SBM-inspired optimization, open-directx bridge) plus
  the communication/persistence layer (AEAD-encrypted, replay-protected
  `SecureChannel`, aruaru-db persistence).
- [`open-web-server`](https://github.com/aon-co-jp/open-web-server) /
  [`RPoem`](https://github.com/aon-co-jp/RPoem) — candidates for the
  coordinator implementation.
- [`aruaru-db`](https://github.com/aon-co-jp/aruaru-db) — where results
  are persisted (ACID-compatible + Git-on-SQL version control).
- [`open-cuda`](https://github.com/aon-co-jp/open-cuda) — GPU execution
  backend.
- [`open-directx`](https://github.com/aon-co-jp/open-directx) — the
  Windows DirectX shader compatibility layer. **Currently the
  top-priority project in this ecosystem for improving completeness.**

## Design document

See [`docs/world-laboratory-design.md`](docs/world-laboratory-design.md)
for the full architecture (a 4-layer design, research into precedents
like BOINC, work-unit design, N-of-M majority-vote verification, Sybil
resistance challenges, and a phased roadmap).

## Current status (2026-08-06, coordinator implemented and verified on real hardware)

`crates/world-lab-coordinator` (Rust + tokio/hyper directly, no Poem
dependency) provides the work-unit distribution / result-submission HTTP
API. It actually uses `dream-os-wire`'s `WorkerChannel`/
`CoordinatorChannel` (AEAD encryption + replay protection), and normal
result submission plus **replay-attack rejection** have been verified
end-to-end with real processes, real HTTP, and real cryptography. N-of-M
majority-vote verification, work-unit persistence, and per-worker key
distribution are not yet implemented (see the HANDOFF section of
[`CLAUDE.md`](CLAUDE.md) for details).

## Related projects

- [aon-co-jp/dream-os](https://github.com/aon-co-jp/dream-os)
- [aon-co-jp/open-directx](https://github.com/aon-co-jp/open-directx)
- [aon-co-jp/open-cuda](https://github.com/aon-co-jp/open-cuda)
- [aon-co-jp/aruaru-db](https://github.com/aon-co-jp/aruaru-db)
- [aon-co-jp/open-web-server](https://github.com/aon-co-jp/open-web-server)
- [aon-co-jp/RPoem](https://github.com/aon-co-jp/RPoem)
- [aon-co-jp/RUNO](https://github.com/aon-co-jp/RUNO) — the ecosystem-wide
  meta index

## License

Undecided (concept stage).
