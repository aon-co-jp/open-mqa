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

**Concept/scope-decision stage only, no code yet** (bootstrapped
2026-08-08).

- Target codecs: FLAC + DSD (software DoP decoding) as the foundation.
- Target quality: DSD256 as of August 2026, with sample-rate/bit-depth
  kept configurable (not hardcoded) to accommodate DSD512 adoption
  expected from 2027 onward.
- "Authentication" equivalent: exploring hash/signature-based proof of
  the mastering chain as an open alternative to MQA's proprietary
  authentication concept.
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
