# open-mqa

Not a reimplementation of MQA (Master Quality Authenticated). Instead, this
project aims to achieve the goal MQA originally served — high-resolution
audio that fits within streaming bandwidth — via an independent pipeline
built on existing open formats (FLAC, DSD256/512), rather than cloning
MQA's proprietary codec.

## Why not an MQA-compatible implementation

MQA's actual encoding/decoding algorithm (the so-called "origami" folding
technique) is patent-protected. Reimplementing it would carry patent
infringement risk separate from copyright concerns. `open-mqa` is
therefore an independent alternative, not a clone of MQA's spec.

## Background on MQA (for context)

- Tidal fully dropped MQA support in July 2024, moving to FLAC.
- MQA Ltd. entered UK administration (roughly equivalent to Chapter 11)
  in April 2023 after losing its main financial backer; Lenbrook
  Industries acquired its assets in September 2023.
- As of 2026-08-08, no official open-sourcing of MQA by the new owner
  has been found.

See [CLAUDE.md](CLAUDE.md) for full sourcing and details.

## Status

**First real code implemented on 2026-08-08** (a Rust FLAC + DoP crate).

- **FLAC encode/decode** (`src/flac.rs`): a thin wrapper delegating to
  existing, well-maintained crates — `claxon` for decoding, `flacenc`
  for encoding. No codec math is hand-rolled. Verified with real
  round-trip tests (mono 16-bit and stereo 24-bit synthetic sine waves,
  encode then decode, sample-for-sample equality) — see build/test
  output below.
- **DoP (DSD over PCM) packing** (`src/dop.rs`): packs/unpacks DSD byte
  streams into 24-bit PCM containers with 0x05/0xFA marker bytes, with
  real unit tests asserting against known byte patterns (round-trip,
  corrupted-marker detection, rejection of odd-length input). Sample
  rate and bit depth are configurable via `DsdFormat`/`DopConfig`, not
  hardcoded.
- **Toshiba SBM (combinatorial optimization) for bit allocation**:
  declined for now — no speculative wiring. See rationale and next
  steps below and in `CLAUDE.md`.

### Build & test

```
cargo build
cargo test
```

As of 2026-08-08, `cargo test` passes all 11 tests (2 FLAC round-trip,
7 DoP packing, 2 input-validation).
- Software fallback design (inspired by DirectX's WARP software
  rasterizer): when native DSD-capable hardware isn't available, fall
  back to a software DSD→PCM downconversion path through the same API
  surface, rather than failing outright.
- On the user's suggestion to incorporate Toshiba's Simulated
  Bifurcation Machine (SBM) and DeepSeek's technology: SBM is real and
  already implemented in `dream-os` (the `sbm_ising` kernel) — reused
  rather than reimplemented, with bit-allocation optimization as a
  candidate future application. The specific claim of "compressing
  thousands of GPUs into one PC" via a DeepSeek "folding" technique
  could **not** be verified via Japanese/English web and GitHub
  research (this matches an earlier, independent finding already
  recorded elsewhere in this ecosystem, on 2026-07-23) — DeepSeek's
  real published techniques (MLA, DeepSeekMoE, FP8 mixed precision,
  compressed attention, Engram memory offloading) are recorded instead
  as the honest basis for any future GPU/memory-efficiency work here.

## Related projects

- [dream-os](https://github.com/aon-co-jp/dream-os) — origin of the sound-tech proposal discussion
- [open-cuda](https://github.com/aon-co-jp/open-cuda) — candidate for future GPU audio DSP integration
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — canonical source for shared dev policy
