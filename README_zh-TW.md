<p align="center">
  <a href="https://ruffle.rs"><img alt="Ruffle" src="https://ruffle.rs/logo.svg" /></a>
</p>
<p align="center">
  <a href="https://github.com/ruffle-rs/ruffle/actions"><img alt="Rust Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_rust.yml?label=Rust%20Build&logo=github&branch=master" /></a>
  <a href="https://github.com/ruffle-rs/ruffle/actions/workflows/test_web.yml"><img alt="Web Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_web.yml?label=Web%20Build&logo=github&branch=master" /></a>
  <a href="https://discord.gg/ruffle"><img alt="Ruffle Discord" src="https://img.shields.io/discord/610531541889581066?label=&color=7389d8&labelColor=6a7ec2&logoColor=ffffff&logo=discord" /></a>
  <a href="https://crowdin.com/project/ruffle"><img alt="Ruffle translations" src="https://badges.crowdin.net/ruffle/localized.svg" /></a>
  <br />
  <a href="https://addons.mozilla.org/en-US/firefox/addon/ruffle_rs/"><img alt="Firefox Extension" src="https://img.shields.io/amo/v/ruffle_rs?logo=firefoxbrowser&logoColor=white&label=firefox" /></a>
  <a href="https://chromewebstore.google.com/detail/ruffle-flash-emulator/donbcfbmhbcapadipfkeojnmajbakjdc"><img alt="Chrome Extension" src="https://img.shields.io/chrome-web-store/v/donbcfbmhbcapadipfkeojnmajbakjdc?logo=googlechrome&logoColor=white&label=chrome" /></a>
  <a href="https://flathub.org/apps/rs.ruffle.Ruffle"><img alt="Ruffle Flathub" src="https://img.shields.io/flathub/v/rs.ruffle.Ruffle?color=007acc&logo=flathub" /></a>
  <a href="https://www.npmjs.com/package/@ruffle-rs/ruffle"><img alt="Ruffle npm" src="https://img.shields.io/npm/v/@ruffle-rs/ruffle?color=007acc&logo=npm" /></a>
  <a href="https://aur.archlinux.org/packages/ruffle-nightly-bin"><img alt="Ruffle AUR" src="https://img.shields.io/aur/version/ruffle-nightly-bin?logo=archlinux" /></a>
  <br />
  <strong><a href="https://ruffle.rs">website</a> | <a href="https://ruffle.rs/demo">demo</a> | <a href="https://github.com/ruffle-rs/ruffle/releases">nightly builds</a> | <a href="https://github.com/ruffle-rs/ruffle/wiki">wiki</a></strong>
</p>

# Ruffle

Ruffle 是一個用 Rust 程式語言編寫的 Adobe Flash Player 模擬器。Ruffle 同時面向桌面端和 Web 端（使用 WebAssembly）。

## 目錄
* [專案狀態](#project-status)
* [使用 Ruffle](#using-ruffle)
* [從原始碼建置](#building-from-source)
  * [前置需求](#prerequisites)
  * [Linux 前置需求](#linux-prerequisites)
  * [桌面端](#desktop)
    * [建置](#build)
    * [macOS](#macos)
  * [Web 或擴充功能](#web-or-extension)
  * [Android](#android)
  * [掃描器](#scanner)
  * [匯出器](#exporter)
* [結構](#structure)
* [贊助者](#sponsors)
* [授權條款](#license)
* [貢獻](#contributing)

## 專案狀態

Ruffle 對 ActionScript 1、2 和 3 支援得相當不錯，但遠未完成。請到 [Issue Tracker](https://github.com/ruffle-rs/ruffle/issues) 回報任何問題。

## 使用 Ruffle

體驗 Ruffle 最簡單的方法是造訪 [Web 示範頁面](https://ruffle.rs/demo/)，然後點擊「Select File」按鈕載入一個你選擇的 SWF 檔案。

[Ruffle 的 Nightly 建置](https://ruffle.rs/downloads#nightly-releases) 可用於桌面端和 Web 端平台。

更詳細的說明請看我們的 [wiki 頁面](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle)。

## 從原始碼建置

### 前置需求

* 最新穩定版的 [Rust](https://www.rust-lang.org/tools/install)
* Java（需要在 PATH 中可用為 `java`）（建置包含 ActionScript 3 內建 Flash 類別的函式庫時需要）

### Linux 前置需求

以下是 Linux 的典型相依項目：

* Ubuntu/Debian：
  ```shell
  sudo apt install pkg-config libasound2-dev libudev-dev libfontconfig-dev libfreetype6-dev libssl-dev default-jre-headless g++
  ```

* Fedora/RHEL：
  ```shell
  sudo dnf install pkgconf-pkg-config alsa-lib-devel systemd-devel fontconfig-devel freetype-devel openssl-devel java-latest-openjdk-headless gcc-c++
  ```

### 桌面端

#### 建置

使用以下命令建置並執行桌面應用程式：

`cargo run --release --package=ruffle_desktop`

要執行某個特定的 SWF 檔案，把 SWF 路徑作為參數傳入：

`cargo run --release --package=ruffle_desktop -- test.swf`

要以除錯模式建置，只需從命令中省略 `--release`。

桌面端有一些選用功能，你可以透過 `--features` 以逗號分隔指定：

* `tracy`：啟用對 [Tracy](https://github.com/wolfpld/tracy) 的分析。你可能需要擴大日誌篩選器才能看到全部內容，例如 `RUST_LOG="warn,ruffle=debug"`
* `tracy_images`：啟用 Tracy 畫面擷取。隱含包含 `tracy`。
* `tracy_avm`：啟用用 Tracy 分析 AVM 執行。隱含包含 `tracy`。
* `render_debug_labels`：渲染時啟用除錯標籤，適合用 [renderdoc](https://renderdoc.org/) 等工具除錯圖形。

#### macOS

Ruffle 桌面端可以透過我們的 [Homebrew Tap](https://github.com/ruffle-rs/homebrew-ruffle/) 建置：

`brew install --HEAD ruffle-rs/ruffle/ruffle`

_注意：由於它是僅 HEAD 版本，每次想要更新時都需要執行 `brew upgrade --fetch-HEAD ruffle`。_

### Web 或擴充功能

按照 [web 目錄中的說明](web/README.md#building-from-source) 建置 Ruffle 的 Web 版或瀏覽器擴充功能版。

本專案使用 BrowserStack 測試。

### Android

按照 [ruffle-android 專案中的說明](https://github.com/ruffle-rs/ruffle-android/blob/main/CONTRIBUTING.md#building-from-source) 建置 Ruffle 的 Android 應用程式。

### 掃描器

如果你有大量「真實世界」的 SWF 用於測試，掃描器可用來評測 ruffle 的解析能力。給它一個資料夾和一個輸出檔名，它會嘗試讀取所有 Flash 檔案並回報解析成功情況。

`cargo run --release --package=ruffle_scanner -- scan folder/with/swfs/ results.csv`

### 匯出器

如果你有 SWF 檔案並想擷取它的圖像，可以使用匯出器工具。這目前需要硬體加速，但可以在無視窗的無頭模式下執行。

- `cargo run --release --package=exporter -- path/to/file.swf`
- `cargo run --release --package=exporter -- path/to/file.swf path/to/screenshots --frames 5`

## 結構

- `core` - 核心模擬器和共用程式碼
- `swf` - SWF 和 ActionScript 解析器
- `desktop` - 桌面用戶端（使用 `wgpu-rs`）
- `web` - Web 用戶端和瀏覽器擴充功能（使用 `wasm-bindgen`）
- `render` - 面向桌面端和 Web 端的多種渲染後端
- `video` - 視訊解碼後端
- `flv` - Flash 視訊解碼器
- `wstr` - 與 Flash 相容的字串實作
- `scanner` - 批次解析 SWF 檔案的工具
- `exporter` - 產生 SWF 檔案 PNG 截圖的工具

## 贊助者

你可以透過 [GitHub Sponsors](https://github.com/sponsors/ruffle-rs) 支援 Ruffle 的開發。你的贊助將有助於確保 Flash 內容在未來的可存取性。感謝！

衷心感謝 Ruffle 的鑽石級贊助者：

<p align="center">
  <a href="https://www.newgrounds.com"><img src="https://ruffle.rs/sponsors/newgrounds.png" alt="Newgrounds.com"></a>
  <a href="https://www.cpmstar.com"><img src="https://ruffle.rs/sponsors/cpmstar.png" alt="CPMStar"></a>
  <a href="https://deepnight.net"><img src="https://ruffle.rs/sponsors/deepnight.png" alt="Sébastien Bénard"></a>
  <a href="https://www.crazygames.com"><img src="https://ruffle.rs/sponsors/crazygames.png" alt="Crazy Games"></a>
  <a href="https://www.coolmathgames.com"><img src="https://ruffle.rs/sponsors/coolmathgames.png" alt="Cool Math Games"></a>
  <a href="https://www.nytimes.com/"><img src="https://ruffle.rs/sponsors/nyt.png" alt="The New York Times"></a>
  <a href="https://www.armorgames.com/"><img src="https://ruffle.rs/sponsors/armorgames.png" alt="Armor Games"></a>
  <a href="https://www.ondaeduca.com/"><img src="https://ruffle.rs/sponsors/ondaeduca.png" alt="Onda Educa"></a>
  <a href="https://www.twoplayergames.org/"><img src="https://ruffle.rs/sponsors/twoplayergames.png" alt="TwoPlayerGames.org"></a>
  <a href="https://www.wowgame.jp/"><img src="https://ruffle.rs/sponsors/wowgame.png" alt="wowgame.jp"></a>
  <a href="http://kupogames.com/"><img src="https://ruffle.rs/sponsors/mattroszak.png" alt="Matt Roszak"></a>
  <a href="https://www.dolldivine.com/"><img src="https://ruffle.rs/sponsors/dolldivine.png" alt="Doll Divine"></a>
  <a href="https://movavi.com/"><img src="https://ruffle.rs/sponsors/movavi.svg" alt="Movavi"></a>
  <a href="https://www.kongregate.com/"><img src="https://ruffle.rs/sponsors/kongregate.svg" alt="Kongregate"></a>
  <a href="https://www.bubbleshooter.net/"><img src="https://ruffle.rs/sponsors/bubble-shooter.png" alt="Bubble Shooter"></a>
  <a href="https://www.neopets.com/"><img src="https://ruffle.rs/sponsors/neopets.png" alt="Neopets"></a>
</p>

## 授權條款

Ruffle 採用以下任一授權條款：

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (http://opensource.org/licenses/MIT)

由你選擇。

Ruffle 依賴相容授權條款下的第三方函式庫。完整資訊見 [LICENSE.md](LICENSE.md)。

### 貢獻

Ruffle 歡迎所有人參與貢獻。開始前請看 [CONTRIBUTING.md](CONTRIBUTING.md)。

除非你另有明確說明，任何你為該專案有意提交的貢獻都將按上述方式雙重授權，不附加任何額外條款或條件。

整個 Ruffle 社群（包括聊天室和 GitHub 專案）都應遵守 Rust 專案本身遵循的《[行為準則](https://www.rust-lang.org/policies/code-of-conduct)》。
