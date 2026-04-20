# Ruffle installer (.msi)
## Prerequisites
To build the msi, you need to set up your environment first:
- [Install `wix`](https://wixtoolset.org/docs/intro/#nettool) (`dotnet tool install --global wix`)
- `wix extension add -g WixToolset.UI.wixext` to add the UI extension
- `wix extension add -g WixToolset.Util.wixext` to add the util extension
- [Build Ruffle desktop](../../../../README.md) for releases (`cargo build --release`)
  - or at least have a prebuilt `exe` ready to go at `target/release/ruffle_desktop.exe` (inside repository root)

## Environment variables
### `RUFFLE_VERSION` (required)
This should be set to the version of Ruffle that this MSI contains.
The format should either be `1.2.3` or `1.2.3.4` - however the fourth value is IGNORED by Windows for "is it the same version or newer" checks.

### `CARGO_BUILD_DIR` (optional)
This should be set to the folder that contains `ruffle_desktop`. The default value is `../../../../target/release`.

# Build
In this directory, run: `wix build ruffle.wxs -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext -arch x64`

You can change `-arch` to `x86` to mark the msi as x86 (and install to, for example, `Program Files (x86)`)

Add `-pdbtype none` to disable generation of the `.wixpdb` if you wish.

Add `-o foo.msi` to control where the MSI is placed.
