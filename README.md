> Latest translations / 翻訳の最新版: [0-LATEST-TRANSLATIONS.md](0-LATEST-TRANSLATIONS.md)

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

### 2026-09-19 訂正 / Correction (FLAC・Opusのレート)

- **FLAC**: 上限96kHzは`flacenc`の検証コードの人為的な制限で、FLAC仕様(フレームヘッダ最大655,350Hz)ではなかった。`vendor/flacenc`(flacenc 0.5.1、Apache-2.0)に1行パッチ(96,000→655,350)を当てて**352.8k/384k/655.35kHz(24bit)の可逆往復を実測で確認**。制約: ①エンコーダは**24bitまで**(32bitはWAV)、②655,350Hz超(705.6k=DSD256のDoP等)はFLAC仕様上不可でWAV専用。DoPはDSD128(352.8kHz)までFLACに載る。
- **Opus**: 規格自体が入力8〜48kHz(最大約20kHz帯域)・本実装は1〜2ch。384kHzは規格上不可能で、Opusにする場合は先に48kHzへ落とす。ハイレゾはWAV/FLACを使う。
- FLAC limit corrected: it was flacenc's own check, patched in `vendor/flacenc` (verified 352.8/384/655.35 kHz at 24-bit). Encoder is 24-bit max; >655,350 Hz (e.g. DSD256 DoP) stays WAV-only. Opus is capped at 48 kHz by the standard itself.

---

作者 / Author: 石 塚 正 浩（MASAHIRO ISHIZUKA） (https://github.com/aon-co-jp)

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
