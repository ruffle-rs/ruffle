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

Ruffle 是一个用 Rust 编程语言编写的 Adobe Flash Player 模拟器。Ruffle 同时面向桌面端和 Web 端（使用 WebAssembly）。

## 目录
* [项目状态](#project-status)
* [使用 Ruffle](#using-ruffle)
* [从源码构建](#building-from-source)
  * [前置要求](#prerequisites)
  * [Linux 前置要求](#linux-prerequisites)
  * [桌面端](#desktop)
    * [构建](#build)
    * [macOS](#macos)
  * [Web 或扩展](#web-or-extension)
  * [Android](#android)
  * [扫描器](#scanner)
  * [导出器](#exporter)
* [结构](#structure)
* [赞助商](#sponsors)
* [许可证](#license)
* [贡献](#contributing)

## 项目状态

Ruffle 对 ActionScript 1、2 和 3 支持得相当不错，但远未完成。请到 [Issue Tracker](https://github.com/ruffle-rs/ruffle/issues) 报告任何问题。

## 使用 Ruffle

体验 Ruffle 最简单的方法是访问 [Web 演示页面](https://ruffle.rs/demo/)，然后点击“Select File”按钮加载一个你选择的 SWF 文件。

[Ruffle 的 Nightly 构建](https://ruffle.rs/downloads#nightly-releases) 可用于桌面端和 Web 端平台。

更详细的说明请看我们的 [wiki 页面](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle)。

## 从源码构建

### 前置要求

* 最新稳定版的 [Rust](https://www.rust-lang.org/tools/install)
* Java（需要在 PATH 中可用为 `java`）（构建包含 ActionScript 3 内置 Flash 类的库时需要）

### Linux 前置要求

以下是 Linux 的典型依赖：

* Ubuntu/Debian：
  ```shell
  sudo apt install pkg-config libasound2-dev libudev-dev libfontconfig-dev libfreetype6-dev libssl-dev default-jre-headless g++
  ```

* Fedora/RHEL：
  ```shell
  sudo dnf install pkgconf-pkg-config alsa-lib-devel systemd-devel fontconfig-devel freetype-devel openssl-devel java-latest-openjdk-headless gcc-c++
  ```

### 桌面端

#### 构建

使用以下命令构建并运行桌面应用：

`cargo run --release --package=ruffle_desktop`

要运行某个特定的 SWF 文件，把 SWF 路径作为参数传入：

`cargo run --release --package=ruffle_desktop -- test.swf`

要以调试模式构建，只需从命令中省略 `--release`。

桌面端有一些可选特性，你可以通过 `--features` 以逗号分隔指定：

* `tracy`：启用对 [Tracy](https://github.com/wolfpld/tracy) 的分析。你可能需要扩大日志过滤器才能看到全部内容，例如 `RUST_LOG="warn,ruffle=debug"`
* `tracy_images`：启用 Tracy 帧捕获。隐式包含 `tracy`。
* `tracy_avm`：启用用 Tracy 分析 AVM 执行。隐式包含 `tracy`。
* `render_debug_labels`：渲染时启用调试标签，适合用 [renderdoc](https://renderdoc.org/) 等工具调试图形。

#### macOS

Ruffle 桌面端可以通过我们的 [Homebrew Tap](https://github.com/ruffle-rs/homebrew-ruffle/) 构建：

`brew install --HEAD ruffle-rs/ruffle/ruffle`

_注意：由于它是仅 HEAD 版本，每次想要更新时都需要运行 `brew upgrade --fetch-HEAD ruffle`。_

### Web 或扩展

按照 [web 目录中的说明](web/README.md#building-from-source) 构建 Ruffle 的 Web 版或浏览器扩展版。

本项目使用 BrowserStack 测试。

### Android

按照 [ruffle-android 项目中的说明](https://github.com/ruffle-rs/ruffle-android/blob/main/CONTRIBUTING.md#building-from-source) 构建 Ruffle 的 Android 应用。

### 扫描器

如果你有大量“真实世界”的 SWF 用于测试，扫描器可用来评测 ruffle 的解析能力。给它一个文件夹和一个输出文件名，它会尝试读取所有 Flash 文件并报告解析成功情况。

`cargo run --release --package=ruffle_scanner -- scan folder/with/swfs/ results.csv`

### 导出器

如果你有 SWF 文件并想捕获它的图像，可以使用导出器工具。这目前需要硬件加速，但可以在无窗口的无头模式下运行。

- `cargo run --release --package=exporter -- path/to/file.swf`
- `cargo run --release --package=exporter -- path/to/file.swf path/to/screenshots --frames 5`

## 结构

- `core` - 核心模拟器和公共代码
- `swf` - SWF 和 ActionScript 解析器
- `desktop` - 桌面客户端（使用 `wgpu-rs`）
- `web` - Web 客户端和浏览器扩展（使用 `wasm-bindgen`）
- `render` - 面向桌面端和 Web 端的多种渲染后端
- `video` - 视频解码后端
- `flv` - Flash 视频解码器
- `wstr` - 与 Flash 兼容的字符串实现
- `scanner` - 批量解析 SWF 文件的工具
- `exporter` - 生成 SWF 文件 PNG 截图的工具

## 赞助商

你可以通过 [GitHub Sponsors](https://github.com/sponsors/ruffle-rs) 支持 Ruffle 的开发。你的赞助将有助于确保 Flash 内容在未来的可访问性。感谢！

衷心感谢 Ruffle 的钻石级赞助商：

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

## 许可证

Ruffle 采用以下任一许可证：

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (http://opensource.org/licenses/MIT)

由你选择。

Ruffle 依赖兼容许可证下的第三方库。完整信息见 [LICENSE.md](LICENSE.md)。

### 贡献

Ruffle 欢迎所有人参与贡献。开始前请看 [CONTRIBUTING.md](CONTRIBUTING.md)。

除非你另有明确说明，任何你为该项目有意提交的贡献都将按上述方式双重许可，不附加任何额外条款或条件。

整个 Ruffle 社区（包括聊天室和 GitHub 项目）都应遵守 Rust 项目本身遵循的《[行为准则](https://www.rust-lang.org/policies/code-of-conduct)》。
