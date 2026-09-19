//! open-mqa: MQA互換を目指さない、特許リスクのない独自ハイレゾ配信パイプライン。
//!
//! 現時点(2026-08-08、初回実装)で提供する機能:
//! - [`wav`]: 自前実装のリニアPCM WAV(16/24/32bit、多チャンネルはEXTENSIBLE)読み書きと、DoPフレームの
//!   24bit WAV入出力。WAVを既定とする。
//! - [`codec`]: 出力形式の選択(**WAV既定**、FLAC/Opusは選択可能、Cargoフィーチャ`flac`/`opus`)。
//! - [`dop`]: DSD-over-PCM (DoP) のマーカー/フレームパッキング。DSD256を
//!   当面の目標品質としつつ、サンプルレート・ビット深度はハードコードせず
//!   呼び出し側が設定可能な構造にしている
//!   (`dream-os/CLAUDE.md`「SOUND関連の技術提案」節の設計方針に準拠)。
//! - [`dsd_modulate`]: PCM→DSDの1次delta-sigma変調器。実際の音声信号
//!   (合成サイン波)から本物の1bit DSDビットストリームを生成し、
//!   [`dop`]のパッキングをE2Eで検証するために追加した(2026-08-08、
//!   既知バイトパターンのみのテストから実信号ベースの検証へ拡張)。
//!
//! 東芝SBM(Simulated Bifurcation Machine)による組み合わせ最適化を用いた
//! ビット配分の統合は、本crateではまだ行っていない。理由と次のステップは
//! リポジトリの`CLAUDE.md`に記録している(投機的な結線を避けるため)。

pub mod dop;
pub mod dsd_modulate;
pub mod codec;
#[cfg(feature = "flac")]
pub mod flac;
#[cfg(feature = "opus")]
pub mod opus;
pub mod wav;
