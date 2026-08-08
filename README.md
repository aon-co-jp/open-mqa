# open-mqa

[English README](README-English.md)

MQA(Master Quality Authenticated)互換の再実装ではなく、MQAが目指していた
「配信帯域に収まる高解像度オーディオ体験」という目的そのものを、既存の
オープンな規格(FLAC・DSD256/512等)を土台に独自パイプラインとして実現する
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
