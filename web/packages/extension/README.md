# ruffle-extension

ruffle-extension is all of the power of Ruffle, in your browser.

Without needing websites to do anything, the browser extension will automatically replace any Flash content on websites
with the Ruffle player.

The extension will automatically negotiate with websites that do have Ruffle installed, to ensure that there is no
conflict between the versions. Newer version of ruffle, either from the website or extension,
will always take precedence and disable the other.

## Using ruffle-extension

The browser extension is built to work in both Chrome and Firefox.

We do not yet have a signed release of the extension, so you must load it as a temporary extension.

Before you can install the extension, you must either download the
[latest build](https://ruffle-rs.s3-us-west-1.amazonaws.com/builds/extension/ruffle_extension_latest.zip)
or [build it yourself](../../README.md).

### Chrome

-   Unpack `dist/ruffle_extension.zip` somewhere
-   Navigate to chrome://extensions/
-   Turn on Developer mode in the top right corner.
-   Click Load unpacked.
-   Select the folder you unpacked the extension to.

### Firefox

-   Navigate to about:debugging.
-   Click on This Firefox.
-   Click Load Temporary Add-on...
-   Select the `.xpi` from the `dist` folder.

## Building, testing or contributing

Please see [the ruffle-web README](../../README.md).
