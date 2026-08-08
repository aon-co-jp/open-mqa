# open-mqa

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

**構想・スコープ決定段階、コード未着手**(2026-08-08新設)。

- 符号化フォーマット: FLAC + DSD(DoP方式)を土台にする方針。
- 目標品質: 2026年8月時点でDSD256、2027年以降のDSD512普及を見据えた
  拡張可能な設計。
- 「認証」相当機能: ハッシュ・署名によるマスタリング工程の証明で代替検討。

## 関連プロジェクト

- [dream-os](https://github.com/aon-co-jp/dream-os) — SOUND関連技術提案の議論の発端
- [open-cuda](https://github.com/aon-co-jp/open-cuda) — 将来のGPU音響DSP連携候補
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本
