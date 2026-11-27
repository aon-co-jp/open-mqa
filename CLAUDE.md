# 設計思想＆開発方針＆開発環境ルール(open-mqa)

> **English summary**: `open-mqa` is not an MQA clone — MQA's actual
> codec ("origami" folding) is patent-protected, so this project
> targets the same goal (bandwidth-efficient hi-res audio) via open
> formats (FLAC, DSD256/512) instead. Concept/scope-decision stage
> only as of 2026-08-08, no code yet. See
> [README-English.md](README-English.md) for the full English summary,
> including the honest finding that DeepSeek's rumored
> "10,000-GPUs-into-one" folding technique could not be verified (same
> conclusion reached independently in this ecosystem back on
> 2026-07-23) — real DeepSeek techniques (MLA, DeepSeekMoE, FP8,
> compressed attention, Engram) are recorded instead.

作業ドライブは`F:\runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の
`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。
GitHubリポジトリ: [aon-co-jp/open-mqa](https://github.com/aon-co-jp/open-mqa)。

**開発開始日: 2026-08-08**(構想・リポジトリ新設日)。

## このプロジェクトの役割

2026-08-08、`dream-os`側のSOUND関連技術提案の議論の中で、ユーザーから
「MQAはオープンソース化で生き残ってもらいましょう。利用手数料が高すぎたり、
経営会社が倒産したりして大変ですね」という提案があり、新規リポジトリとして
着手した。

**重要な方針決定(ユーザー選択、2026-08-08)**: MQA社の実際のエンコード/
デコードアルゴリズム(通称「折り紙(origami)」技術、時間領域の音声情報を
折りたたんでPCMコンテナへ格納する独自方式)は特許で保護されており、
これを再実装することは著作権とは別に**特許侵害のリスク**を伴う。
そのため`open-mqa`は**MQA互換(ビット完全なデコード実装)を目指さない**。
代わりに、**MQAが目指していた「配信帯域に収まる高解像度オーディオ体験」
という目的そのものを、既存のオープンな規格(FLAC・DSD256/512等)を土台に
した独自パイプラインとして実現する**ことを目指す(選択肢
「MQA互換ではなく独自のハイレゾ技術」を採用、選択肢「MQA仕様の再実装を
試みる」〈特許侵害リスクあり〉・「調査のみ」は不採用)。

## MQAの経緯(調査済み、`dream-os/CLAUDE.md`の同日エントリと重複記録)

- MQA(Master Quality Authenticated)は英MQA Ltd.が開発した高解像度
  オーディオの符号化・認証方式。主要ストリーミングサービスTidalが
  2024年7月に対応を完全に打ち切りFLACへ移行済み
  ([Audioengine](https://audioengine.com/explore/tidal-announces-they-are-dropping-support-for-mqa-hi-res-audio/))。
- MQA Ltd.は2023年4月6日に英国の「administration」(Chapter 11相当の
  会社更生手続き)を申請、主要な資金提供者の撤退が引き金
  ([What Hi-Fi?](https://www.whathifi.com/features/mqa-has-gone-into-administration-what-does-this-mean-for-tidal-and-supported-products))。
  同年9月にカナダのLenbrook Industriesが資産を買収
  ([Strata-gee.com](https://www.strata-gee.com/mqa-limited-no-longer-exists-say-hello-to-wave-realisations-limited/))。
- **2026-08-08時点、Lenbrook/Wave Realisations社がMQAを公式にオープン
  ソース化したという発表は確認できていない**——`open-mqa`はMQA社自身の
  オープンソース化を代行・先取りするものではなく、あくまで独立した
  「MQAが目指した目的への、特許リスクのない別解」という位置づけ。

## スコープ(独自ハイレゾパイプライン、2026-08-08時点の初期方針)

- **符号化フォーマット**: FLAC(既に業界標準、オープンソース、
  ロイヤリティフリー)・DSD(DoP方式でのソフトウェアデコード)を土台と
  する。`dream-os`側の調査結果([dream-os/CLAUDE.md](https://github.com/aon-co-jp/dream-os/blob/main/CLAUDE.md)
  「SOUND関連の技術提案」節参照)に合わせ、**2026年8月8日時点の目標品質は
  DSD256**、**2027年以降のDSD512普及を見据えてサンプルレート/ビット深度を
  ハードコードしない設計**とする。
- **「認証(Authenticated)」に相当する要素**: MQAの「スタジオでの
  マスタリング工程を認証する」というコンセプト自体はオープンな手段
  (例: 音源のハッシュ値・メタデータ署名)で代替可能と考えられる
  (詳細設計は次回以降)。
- **配信帯域最適化**: MQAが売りにしていた「ハイレゾ音源を低帯域で
  配信する」という価値提案自体は、既存のロスレス圧縮(FLAC)+適応的
  ビットレート配信の組み合わせで大部分をカバーできると考えられる
  (MQA独自の可逆的ダウンサンプリング〈折り紙技術〉のような専売特許の
  手法は使わない)。
- **GPU/NPU実装**: `dream-os`の提案通り、GPUオーディオDSP
  ([GPU Audio](https://www.gpu.audio/)のような実在する業界動向)・
  `open-cuda`(Vulkan計算基盤)との連携も将来検討する。

## PCM代替のソフトウェアフォールバック構想(2026-08-08、ユーザー提案+日英Web検索での裏取り)

ユーザー提案「PCMは次のハードウェアがない場合のDirectXなどの代替技術で」
を検討・裏取りした。

- **実在する裏付け**: DirectXの`WARP`(Windows Advanced Rasterization
  Platform)は、対応GPUハードウェアが無い場合にCPU側でソフトウェア
  ラスタライズへフォールバックする仕組み
  ([Wikipedia](https://en.wikipedia.org/wiki/Windows_Advanced_Rasterization_Platform))。
  音響分野にも同型のパターンが実在する: **DSD対応DACを持たない
  ハードウェアでは、再生ソフトウェア側でDSDをPCMへダウンコンバート
  して再生する**のが標準的な対処法
  ([NativeDSD Help](https://help.nativedsd.com/en/articles/63529-playing-dsd-dxd-and-very-high-bit-rate-pcm-files)、
  [Audiophile Style](https://audiophilestyle.com/forums/topic/38313-converting-dsd-to-pcm/))。
  変換自体は1bit DSD→64bit PCM(DSDレートの1/8)への実質ロスレス変換だが、
  折り返し歪み(スペクトラムフォールディング)を避けるためデジタル
  フィルタ(シグマデルタ復調)が必須
  ([samplerateconverter.com](https://samplerateconverter.com/educational/dsd-converter))。
- **open-mqaへの設計方針**: WARPと同じ「ハードウェアが対応していれば
  ネイティブ実行、無ければソフトウェアフォールバック」という抽象化を
  音響パイプラインにも適用する——具体的には、DSD256/512ネイティブ
  対応DACがあればそのまま出力し、無ければソフトウェア側でPCM
  (24bit/192kHz等)へダウンコンバートしてから出力する経路を、
  同じAPI面から透過的に選択できる設計とする(次回、具体的なインター
  フェース設計に着手)。

## 東芝SBM・DeepSeek技術の組み込み検討(2026-08-08、ユーザー提案+日英Web検索・GitHub調査での裏取り)

ユーザー提案「東芝の疑似量子コンピュータ技術とDeepSeekのグラフィック
ボード一万枚を一枚でPCで実現する折りたたみ技術を盛り込んで」を検討した。

- **東芝SBM(Simulated Bifurcation Machine、疑似量子アニーリング)**:
  実在する技術で、**`dream-os`側に既に実装済み**(`sbm_ising`カーネル、
  64スピンPoC、実GPU/NPU上で動作検証済み——
  [dream-os/CLAUDE.md](https://github.com/aon-co-jp/dream-os/blob/main/CLAUDE.md)
  参照)。組み合わせ最適化問題(イジングモデルへ定式化できる問題)を
  高速に解く技術であり、**音響分野での現実的な適用先候補**としては、
  「限られたビットレート予算の中で、心理音響モデルに基づき周波数
  帯域ごとの最適なビット配分を組み合わせ最適化問題として解く」
  ような用途が考えられる(既存のFLAC/MP3等のロスレス/非可逆圧縮も
  内部でビット配分の最適化を行っており、SBMで置き換え可能かは
  次回の技術調査対象)。**再実装はせず、`dream-os`の既存実装を
  再利用する方針**とする(このエコシステムの既存方針「他リポジトリが
  既に持つ機能を重複実装しない」に従う)。
- **DeepSeekの「グラフィックボード一万枚を一枚でPCで実現する折りたたみ
  技術」について(重要な正直な開示)**: **2026-08-08時点で日英Web検索・
  GitHub調査を行ったが、そのような技術は確認できなかった**。これは
  今回が初めての調査ではなく、**このエコシステム内で2026-07-23時点
  (`open-web-server`/`open-directx`側CLAUDE.md参照)に既に同じ主張を
  調査済みで、当時も「数千枚のGPUを1枚に圧縮する技術という主張は
  確認できず、誤解・誇張と判断」という結論に達している**——今回の
  再調査でも同じ結論に至った(2度目の裏取りで再確認)。
  DeepSeekが実際に発表している技術は以下の通り(いずれも「複数GPUの
  計算能力を1枚に圧縮する」話ではなく、**メモリ効率・通信最適化・
  アテンション機構の圧縮**が中心):
  - **MLA(Multi-Head Latent Attention)**・**DeepSeekMoE**・
    **FP8混合精度学習**(DeepSeek-V3、既存調査で確認済み)。
  - **Compressed Sparse Attention / Heavily Compressed Attention**
    (DeepSeek-V4のハイブリッドアテンション、KVキャッシュのメモリ
    フットプリント削減、["DeepSeek: Paradigm Shifts and Technical
    Evolution in Large AI Models"](https://arxiv.org/pdf/2507.09955))。
  - **Engram**(長文脈クエリ向けに、静的な知識をシステムRAMへコミット
    しHBM依存を減らすメモリアーキテクチャ、
    [Tom's Hardware](https://www.tomshardware.com/tech-industry/artificial-intelligence/deepseek-touts-memory-breakthrough-engram))
    ——**これは「GPU/HBM制約を回避するためシステムRAMを併用する」という
    方向性であり、方向性としては「限られたハードウェアで大規模モデルを
    動かす」というユーザーの意図に最も近い実在技術**と考えられる。
  - **DeepSeek-OCR**(光学2次元マッピングによるコンテキスト圧縮、
    10倍圧縮でOCR精度97%、単一A100 GPUで1日20万ページ超処理、
    [deepseek.ai](https://deepseek.ai/blog/deepseek-ocr-context-compression))。
  - **Warp specialization**(Streaming Multiprocessorを通信チャネルへ
    分割する最適化、[Medium記事](https://medium.com/@amin32846/unlock-warp-level-performance-deepseeks-practical-techniques-for-specialized-gpu-tasks-a6cf0c68a178))。
  **open-mqaへの現実的な適用方針**: 「1万枚を1枚に折りたたむ」という
  文字通りの技術は実装対象にせず(存在しないため)、代わりに**Engram
  的なメモリオフロード方針(GPU VRAM不足時にシステムRAMを併用する
  設計)**を、将来GPU実装(GPU Audio的な音響DSP)に予算制約がある環境
  向けの現実的な代替案として記録しておく。
- **正直な結論**: DirectX・未来のOS/CPU/NPU/GPU実装レベルへの期待
  自体は方向性として理解できるが、**現時点で存在しない技術を
  「盛り込んだ」と主張することはこのエコシステムの正直な開示の方針に
  反する**——実在する東芝SBM(dream-os経由で再利用)・DeepSeekの実際の
  メモリ効率化技術(Engram的な発想)を、それぞれ妥当な適用先が見つかった
  時点で段階的に検討する、という現実路線を維持する。

**現時点ではコード未着手、構想・スコープ決定のみ**(このCLAUDE.md・
README.md・PORTING.mdの新設が今回の唯一の成果物)。

## 運用ルール

`open-raid-z`側の全リポジトリ共通ルール(比較的新しい技術の参照資料一覧・
AI駆動開発ツールに関する所感・確認不要の自動継続・白画面バグ等を見逃さない
検証徹底、等)に準じる。詳細は`open-raid-z/CLAUDE.md`を参照。

**コミット・push方針(2026-08-08新設)**: このリポジトリはこれまでの
セッションで一貫して全コミットをそのままpushしてきた実績があるため、
今後も**変更をコミットした場合は都度originへpushする**ことを既定運用と
する(空リポジトリの立ち上げ段階でこの方針が未記載だったため、ここで
明文化)。

## FLAC/DoP実装(2026-08-08、最初の実コード)

`src/flac.rs`・`src/dop.rs`として、Rust製の最初の実装crateを追加した
(詳細はREADME.md/README-English.mdを参照)。

- **FLAC**: `claxon`(デコード)・`flacenc`(エンコード)という既存の
  実績あるcrateへ委譲する薄いラッパー。コーデック内部の予測・
  エントロピー符号化を自前実装することはしていない。
  - **実装中に踏んだ落とし穴(正直な記録)**: `flacenc::encode_with_fixed_block_size`
    は入力サンプル数が`block_size`の倍数でない場合、末尾ブロックを
    ゼロ詰めして固定長ブロックとしてエンコードする。この詰め物は
    ビットストリーム上に実際に存在するフレームとして残るため、
    `claxon`は素直にゼロ詰め分まで含めてデコードしてしまい、往復
    テストの最初の実行では末尾に余分なゼロサンプルが混入して失敗した
    (`round_trip_mono_sine_16bit`: 期待4410サンプルに対し実際8192
    サンプル)。対処として、`decode_flac`側でSTREAMINFOの
    `samples`(真の総フレーム数)を読み取り、そこへ切り詰める処理を
    追加して解決(`src/flac.rs`の`decode_flac`内、`total_frames`変数)。
- **DoP**: DSDバイト列を0x05/0xFAマーカー付き24bit PCMコンテナへ
  パッキング/アンパッキングする実装。サンプルレート・ビット深度は
  `DsdFormat`/`DopConfig`構造体でパラメータ化し、DSD64/128/256/512の
  いずれにも対応可能(ハードコードなし、`dream-os/CLAUDE.md`
  「SOUND関連の技術提案」節の設計方針に準拠)。
- **検証**: `cargo build`・`cargo test`を実行し、実際に成功したことを
  確認済み(下記HANDOFF参照、コマンド出力の要約を記録)。実FLACテスト
  ファイルは用意していないが、その代替として合成サイン波(モノラル
  16bit・ステレオ24bit)をエンコード→デコードし、サンプル値が完全一致
  することをテストで検証した(フェイクのアサーションではない)。

## 東芝SBMによるビット配分最適化(2026-08-08、統合可否の検討結果)

`dream-os`側`sbm_ising`モジュール(`crates/dream-os-kernel/src/sbm.rs`、
`run_sbm_ising`/`run_sbm_ising_cpu_reference`)のAPIを確認した上で、
FLACエンコードのビット配分(ブロックサイズ・予測次数選択)への統合可否を
検討した。

- **結論: 現時点では統合しない(投機的な結線を避ける)**。理由:
  FLACのブロックサイズ/予測次数選択は、実務上はエンコーダ側で複数候補を
  試して残差が最小になるものを選ぶ**貪欲法・全探索に近い最適化**で
  行われており、イジングモデル(スピン間の相互作用を表す`J`行列)へ
  自然に定式化できる「組み合わせ最適化問題」としての形が見えていない。
  無理に定式化(例: ブロックサイズ候補の二値選択をスピンに割り当てる等)
  することは可能だが、現時点でそれがSBMの強み(大規模な組み合わせ空間を
  高速に探索する)を活かせる具体的な問題設定になっているとは言えず、
  「SBMを使いたいから無理に当てはめた」というだけの実装になるリスクが
  高いと判断した。
  - **再実装はしない方針は維持**: 将来、具体的で妥当な適用先
    (例: 複数チャンネル・複数周波数帯域にまたがるビットレート予算配分の
    ような、より大規模で真に組み合わせ的な問題)が見つかった場合は、
    `dream-os-kernel`crateへのpath dependency経由で`run_sbm_ising`
    (または`run_sbm_ising_cpu_reference`、GPU無しの環境向け)を再利用する。
  - **次回の具体的な調査ステップ**(ロードマップ): (1) 単一ファイルの
    ブロックサイズ選択ではなく、**アルバム単位・プレイリスト単位で
    複数曲にまたがる配信ビットレート予算の配分**(帯域制約下で全体の
    音質劣化を最小化する組み合わせ問題)であれば、スピン=各曲への
    ビットレート階級割り当てとして定式化できる可能性がある。この方向で
    小さなプロトタイプ(数曲・数階級程度)を作り、貪欲法との音質/計算
    時間の比較実験を行うことを次回の検討対象とする。(2) それが有望
    でなければ、SBM統合は見送ったままFLAC/DoPパイプラインの機能拡張
    (「認証」相当機能の実装等)を優先する。

## DeepSeek「GPU一万枚折りたたみ」技術(再確認不要、既に決着済み)

本日(2026-08-08)このリポジトリ自身のCLAUDE.md冒頭・「東芝SBM・DeepSeek
技術の組み込み検討」節で既に調査済み・「確認できず」という結論に到達して
いる(2026-07-23の`open-web-server`/`open-directx`側調査と合わせて2度目の
確認)。**今後のセッションでこの主張を三度目以降調査する必要はない**——
本節および同ファイル上部の既存記述を参照すれば足りる。

## 関連プロジェクト

- **dream-os**: SOUND関連技術提案の議論の発端、DSD256/512の目標品質設定を
  共有: https://github.com/aon-co-jp/dream-os
- **open-cuda**: 将来のGPU音響DSP連携候補: https://github.com/aon-co-jp/open-cuda
- **open-raid-z**: 開発ルールの正本: https://github.com/aon-co-jp/open-raid-z

## HANDOFF(直近の作業ログ、上が最新)

### 2026-08-08 (続き2) 実DSDビットストリームによるDoPパッキングのE2E検証(「次にすべきこと(3)」対応)

前回HANDOFFの「次にすべきこと(3): 実際のDSDバイナリ(1bitストリーム)を
用いたDoPパッキングのE2E検証(現状は既知バイトパターンによる単体テスト
のみ)」に対応した。

- **実装**: 新規`src/dsd_modulate.rs`——1次delta-sigma変調器
  (`DeltaSigmaModulator`)。PCMサンプル列(f32)から本物の1bit DSD
  ビットストリームをMSBファーストでバイトへパッキングする
  `modulate_to_bytes()`を実装。高次NTF(ノイズシェーピング多項式)等の
  高品質化は行わない最小実装(過剰実装回避)。
- **E2Eテスト2件を追加**: (1)
  `real_dsd_bitstream_round_trips_through_dop_packing`——合成サイン波
  (1kHz、DSD256サンプルレート相当)を実際に1次ΔΣ変調しDSDビット
  ストリームを生成、これを`pack_dop_frames`→`unpack_dop_frames`で
  往復させバイト単位で完全一致することを確認。既知バイトパターンでは
  なく実信号由来のバイト列を通した検証に格上げした。(2)
  `demodulated_signal_correlates_with_original_pcm`——DoP往復だけでは
  「パッキング層が壊していないか」しか検証できないため、加えて
  「変調後のDSDビット列が実際に元信号を表現しているか」を、簡易移動
  平均復調(`crude_demodulate`、テスト専用の最小ローパス)→ピアソン
  相関係数で確認。実測相関は0.7超の閾値を余裕を持って超えることを
  確認済み(実装がノイズを出しているだけではないことの実証)。
- **検証**: `cargo test`は**全14件green**(既存12件+新規2件、
  リグレッション無し)。
- **正直な開示・スコープ外**: (1) `crude_demodulate`はテスト検証専用の
  簡易矩形窓移動平均であり、実DACのマルチビットノイズシェーピング
  復調フィルタの代替ではない。(2) 1次ΔΣは高次ΔΣ(実際のDSDエンコーダ
  が使う3次〜7次程度のノイズシェーピング)と比べ量子化ノイズの
  高域抑圧が弱い——音質面での実用性は今回のスコープ外(あくまで
  「DoPパッキングが実DSDデータを正しく往復させる」ことの実証が目的)。
  (3) 実際のオーバーサンプリング(PCM→高サンプルレートへのアップ
  サンプリング)は行っていない——テストでは変調器に直接DSDレートの
  サンプルを供給している(実運用では別途リサンプラーが必要、次回
  以降の課題として残す)。
- 次にすべきこと: (1) 高次ΔΣ(ノイズシェーピング)への拡張、
  (2) PCM→DSDレートへのオーバーサンプリング/リサンプラー実装、
  (3) 前回HANDOFFに記載の残り項目(SBMビットレート予算配分プロト
  タイプ、認証相当機能、VPS `/root/open-mqa`新設)。

### 2026-08-08 (続き) 最初の実装: FLAC/DoP crate作成・ビルド/テスト検証

構想段階から前進し、Rustプロジェクトを新規作成(`Cargo.toml`・
`src/lib.rs`・`src/flac.rs`・`src/dop.rs`)。

- **実装内容**: 上記「FLAC/DoP実装」節・「東芝SBMによるビット配分
  最適化」節を参照。
- **実ビルド・テスト結果(正直な開示、実際に実行したコマンドの出力)**:
  - `cargo build` → `Finished \`dev\` profile [unoptimized + debuginfo]
    target(s) in 3.47s`(成功、依存crateは`claxon 0.4.3`・
    `flacenc 0.4.0`・`thiserror 1.0.69`をcrates.ioから取得)。
  - `cargo test` → `test result: ok. 11 passed; 0 failed; 0 ignored;
    0 measured; 0 filtered out`(全11件pass)。内訳: FLAC往復一致
    テスト2件(モノラル16bit・ステレオ24bitの合成サイン波)、DoP
    パッキングテスト7件(既知バイトパターン一致・往復一致・マーカー
    破損検知・空入力・DSD256/512のビットレート定数確認等)、入力検証
    テスト2件(チャンネル長不一致・奇数長DSDバイト列の拒否)。
  - 実装途中で`flacenc::encode_with_fixed_block_size`が末尾ブロックを
    ゼロ詰めする挙動により最初のテスト実行が失敗した経緯・修正内容は
    上記「FLAC/DoP実装」節に記録(隠さず記載)。
  - 実FLACテストファイル(市販音源等)は用意していない
    ——合成サイン波によるエンコード→デコード往復一致という、実データを
    使った検証で代替した(フェイクのアサーションではない)。
- **見送った項目(正直な開示)**: (1) DeepSeekの「GPU一万枚折りたたみ」
  技術の再調査は行っていない——本日この`CLAUDE.md`内で既に2度目の
  裏取りが完了し「確認できず」という結論が出ているため(上記
  「DeepSeek『GPU一万枚折りたたみ』技術」節参照)、三度目の調査は
  重複作業と判断し実施しなかった。(2) 東芝SBM(`sbm_ising`)を
  FLACのビット配分へ結線することは見送った——現状のFLACブロック
  サイズ/予測次数選択は貪欲法的な最適化で十分に扱えており、イジング
  モデルへ自然に定式化できる具体的な問題設定が見えていないため
  (詳細は上記節、投機的な結線を避けるという方針に従う)。
- **新規運用ルール**: 「コミットしたら都度push」を本リポジトリの
  既定方針として明文化(上記「運用ルール」節)。この方針に従い、
  今回の変更もコミット後にpush済み(pushの成否は次回セッションの
  冒頭で`git log`/`git status`により再確認可能)。
- **次にすべきこと**: (1) 上記SBMロードマップ(アルバム/プレイリスト
  単位のビットレート予算配分プロトタイプ)の検討。(2) 「認証」相当
  機能(ハッシュ・署名によるマスタリング工程の証明)の設計・実装。
  (3) 実際のDSDバイナリ(1bitストリーム)を用いたDoPパッキングの
  E2E検証(現状は既知バイトパターンによる単体テストのみ)。(4) VPS
  (conoha)側`/root/open-mqa`フォルダの新設(実装が今回進んだため、
  次回セッションでデプロイ方針を検討してよい段階に入った)。

### 2026-08-08 リポジトリ新規作成・構想文書化

ユーザー指示「open-mqaリポジトリVPSとローカルドライブフォルダも作って」
「MQAはオープンソース化で生き残ってもらいましょう」を受け、新規リポジトリ
`aon-co-jp/open-mqa`をGitHub API経由で作成、ローカル
`F:\runo\open-mqa`へclone。事前にユーザーへ命名・配置・スコープ
(MQA互換 vs 独自技術 vs 調査のみ)を確認し、「MQA互換ではなく独自の
ハイレゾ技術」を選択いただいた上で着手。

- **正直な開示**: 現時点でコードは一切無い、構想・スコープ決定のみの
  段階(`open-directx`/`dream-os`が2026-07-01/07-25/08-06に辿ったのと
  同じ「空リポジトリ→まず構想文書化」というこのエコシステムの既定
  パターンに従う)。
- VPS(conoha)側の`/root/open-mqa`はまだ作成していない(次回、実際に
  デプロイするコードができてから新設する方針——他リポジトリも実装が
  先行してからVPS側フォルダを作る運用が一般的なため)。
- 次にすべきこと: (1) FLAC/DSD256ベースの独自符号化パイプラインの
  技術調査(既存OSSライブラリ〈libFLAC等〉の再利用可否含む)、
  (2) 「認証」相当機能(ハッシュ・署名によるマスタリング工程の証明)の
  設計、(3) 最初の1機能(例: FLACエンコード/デコードの薄いラッパー)の
  実装着手、(4) 実装が固まった段階でVPSフォルダを新設しデプロイ。

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

### 2026-09-19 訂正 / Correction (FLAC・Opusのレート)

- **FLAC**: 上限96kHzは`flacenc`の検証コードの人為的な制限で、FLAC仕様(フレームヘッダ最大655,350Hz)ではなかった。`vendor/flacenc`(flacenc 0.5.1、Apache-2.0)に1行パッチ(96,000→655,350)を当てて**352.8k/384k/655.35kHz(24bit)の可逆往復を実測で確認**。制約: ①エンコーダは**24bitまで**(32bitはWAV)、②655,350Hz超(705.6k=DSD256のDoP等)はFLAC仕様上不可でWAV専用。DoPはDSD128(352.8kHz)までFLACに載る。
- **Opus**: 規格自体が入力8〜48kHz(最大約20kHz帯域)・本実装は1〜2ch。384kHzは規格上不可能で、Opusにする場合は先に48kHzへ落とす。ハイレゾはWAV/FLACを使う。
- FLAC limit corrected: it was flacenc's own check, patched in `vendor/flacenc` (verified 352.8/384/655.35 kHz at 24-bit). Encoder is 24-bit max; >655,350 Hz (e.g. DSD256 DoP) stays WAV-only. Opus is capped at 48 kHz by the standard itself.

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.

## 完成まで作り切る方針 / Finish everything policy (2026-11-27)
**日本語**: 全リポジトリで、**未確認は自動で確認し、未実施は実施し、未了は完成させる**(確認を求めず進める)。使用頻度が低いと判断される機能や、**1から作り直す大規模開発になる内容でも、やめずに最後まで開発・TEST・DEBUGする**(別の方式で済ませない、後回しにしない)。進め方: 調査(英語・日本語の検索とGitHub) → 設計 → 実装 → 実機/実データで確認 → 失敗は原因を直して再試験 → README・CLAUDE・PORTING・POLICY を日英で更新 → push。「できた」と書くのは実測した範囲だけ、確認できなかった所は「未確認」と正直に書く。例外(確認を取る): 元に戻せない削除、秘密の公開、所有者の認証・決済・実機操作が要るもの、利用者のシステム設定の変更。
**English**: In every repository, **verify what is unverified, do what is not yet done, and finish what is unfinished** automatically, without asking. Even features that look rarely used, or work that means **a large rewrite from scratch, are developed, tested and debugged to the end** (no substituting another approach, no postponing). Method: research (English and Japanese search plus GitHub) -> design -> implement -> verify on real devices or data -> fix causes and retest on failure -> update README, CLAUDE, PORTING and POLICY in Japanese and English -> push. Write "done" only for what was measured, and say "unverified" honestly for the rest. Exceptions that still need confirmation: irreversible deletions, publishing secrets, anything needing the owner's authentication, payment or real-device actions, and changes to the user's system settings.