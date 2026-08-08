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

## 関連プロジェクト

- **dream-os**: SOUND関連技術提案の議論の発端、DSD256/512の目標品質設定を
  共有: https://github.com/aon-co-jp/dream-os
- **open-cuda**: 将来のGPU音響DSP連携候補: https://github.com/aon-co-jp/open-cuda
- **open-raid-z**: 開発ルールの正本: https://github.com/aon-co-jp/open-raid-z

## HANDOFF(直近の作業ログ、上が最新)

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
