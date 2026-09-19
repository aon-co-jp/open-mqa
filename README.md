# open-mqa

[English README](README-English.md)

MQA(Master Quality Authenticated)互換の再実装ではなく、MQAが目指していた
「配信帯域に収まる高解像度オーディオ体験」という目的そのものを、既存の
オープンな規格(WAV・DSD256/512等)を土台に独自パイプラインとして実現する
プロジェクト。

## なぜMQA互換を目指さないか

MQA社の実際のエンコード/デコードアルゴリズム(通称「折り紙」技術)は特許で
保護されており、これを再実装することは著作権とは別に特許侵害のリスクを
伴う。`open-mqa`はMQA仕様のクローンではなく、独立した別解として設計する。

## MQAを取り巻く経緯(参考)

- Tidalが2024年7月にMQA対応を完全に打ち切りFLACへ移行。
- MQA Ltd.は2023年4月に英国のadministration(Chapter 11相当)を申請、
  同年9月にLenbrook Industriesが資産を買収。
- 2026-08-08時点、買収先によるMQAの公式オープンソース化の発表は
  確認できていない。

詳細・出典は[CLAUDE.md](CLAUDE.md)を参照。

## 現状

**2026-08-08、最初の実コードを実装済み**(Rust製FLAC+DoP crate)。

- **FLACエンコード/デコード**(`src/flac.rs`): `claxon`(デコード)と
  `flacenc`(エンコード)という既存の実績あるcrateへ委譲する薄いラッパー。
  コーデック内部の数学処理は自前実装していない。サイン波によるモノラル
  16bit/ステレオ24bitのエンコード→デコード往復一致テストで実際に検証済み
  (`cargo test`で確認、下記参照)。
- **DoP(DSD over PCM)パッキング**(`src/dop.rs`): DSDバイト列を
  0x05/0xFAマーカー付き24bit PCMコンテナへ詰める/戻す実装。既知のバイト
  パターンに対するアサーション付きの実テストあり(往復一致・マーカー
  破損検知・奇数長入力の拒否など)。サンプルレート・ビット深度は
  ハードコードせず、`DsdFormat`/`DopConfig`で設定可能。
- **東芝SBM(組み合わせ最適化)によるビット配分への統合**: 見送り
  (投機的な結線はしない方針)。理由と今後の調査方針は本README下部・
  `CLAUDE.md`を参照。

### ビルド・テスト

```
cargo build
cargo test
```

2026-08-08時点で`cargo test`は11件全てpass(FLAC往復一致2件、DoP
パッキング7件、入力検証系2件)。

## 関連プロジェクト

- [dream-os](https://github.com/aon-co-jp/dream-os) — SOUND関連技術提案の議論の発端
- [open-cuda](https://github.com/aon-co-jp/open-cuda) — 将来のGPU音響DSP連携候補
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本

## 2026-09-19 方針変更: FLAC廃止・WAVベースへ / Switched from FLAC to WAV

FLAC(`claxon`/`flacenc`依存)を廃止し、自前実装のWAV(`src/wav.rs`、16/24/32bit・多ch対応)を基本形式にした。DoPフレームも24bit WAV(`encode_dop_wav`/`decode_dop_wav`)として保存でき、DoPマーカー込みのビット完全な往復をテストで確認済み(全17件pass)。`make-disk`(F:\make-disk)のDSD/ハイレゾ変換とWAVで受け渡せる。**正直な開示**: DoP WAVは対応DACへのビットパーフェクト再生が前提(音量・SRCを通るとノイズ)。MQA互換ではなく、MQAの再実装も行わない。 / FLAC (and its crate deps) removed in favour of a self-written WAV reader/writer; DoP frames round-trip bit-exactly through 24-bit WAV. Playable only via bit-perfect paths to DoP-capable DACs. Still not MQA-compatible.

## 2026-09-19 関連リポジトリ取り込み判断 / Sibling-repo integration decision

- **open-cuda(採用候補・今回はコード結線せず)**: `hgemm`/`dgemm`/`sgemm`の実Vulkan実行(GT 730実機で検証済み)があり、オーバーサンプリング用FIRの行列化など**GPU向きの並列処理**には将来使える。ただしΔΣ変調は出力ビットを次サンプルの誤差へ戻す**直列フィードバック**でGPUに向かない(並列化できるのはチャンネル間・区間間のみ。`make-disk`側で区間並列化を実装予定)。open-cudaも生成系ではGPUディスパッチが実用速度に届いていないため、投機的な結線はしない方針を維持。
- **open-directx(見送り)**: 画像/動画コーデック(FFv1等)が中心で、音声パイプラインとの接点が無い。
- **aruaru-llm(見送り)**: LLM基盤で、音声信号処理との接点が無い。
- 実際にDSD/ハイレゾ変換を行う実装は`make-disk`(`src-tauri/src/engine/dsd.rs`)側にあり、open-mqaはWAV/DoPの受け渡し形式を担う。 / Only open-cuda is a plausible future fit (GPU-friendly FIR/oversampling), not wired now because ΔΣ is serial and open-cuda's GPU path isn't at practical speed; open-directx and aruaru-llm have no audio overlap.

## 2026-09-19 出力形式: WAV既定、FLAC/Opusは選択制 / Output formats: WAV default, FLAC/Opus selectable

`codec::AudioFormat`(既定=`Wav`)で形式を選ぶ。FLACとOpusはCargoフィーチャ`flac`/`opus`(既定で有効、`--no-default-features`で外せる)。
- **WAV**: 無圧縮、16/24/32bit・多ch・任意レート(DoPも可)。
- **FLAC**(`flacenc`/`claxon`): 可逆。**実測の制約: このエンコーダは96kHzまで**。DoP(176.4kHz以上)は載せられずWAV専用。
- **Opus**(純Rust`opus-pure`、C/cmake不要): 非可逆。8/12/16/24/48kHz・1〜2chのみでハイレゾ/DoP用ではない(DoPは明示的に拒否)。ギャップレス(元と同じ長さ)を往復テストで確認。
- `codec::decode`はマジックバイトでWAV/FLAC/Opusを自動判別。全26テスト+`--no-default-features`ビルド成功。

WAV is the default; FLAC (≤96 kHz with flacenc, lossless) and Opus (pure-Rust, lossy, ≤48 kHz stereo) are opt-in formats. DoP is WAV-only (FLAC's encoder caps at 96 kHz; Opus would destroy it).
