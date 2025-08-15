# Ruffle Bundle (.ruf) format specification
A Ruffle Bundle is an easy way to package and share Flash games and any assets that are required to make the game work.

A bundle can be a directory or a renamed zip file, and must contain at minimum a `ruffle-bundle.toml` file.

<!-- TOC -->
* [Ruffle Bundle (.ruf) format specification](#ruffle-bundle-ruf-format-specification)
  * [Directory structure](#directory-structure)
    * [`ruffle-bundle.toml` (Bundle information)](#ruffle-bundletoml-bundle-information)
    * [`content/` (Flash content)](#content-flash-content)
  * [`ruffle-bundle.toml` file specification](#ruffle-bundletoml-file-specification)
    * [`[bundle]`](#bundle)
      * [`name` - The name of the bundle](#name---the-name-of-the-bundle)
      * [`url` - The url of the Flash content to open](#url---the-url-of-the-flash-content-to-open)
    * [`[player]`](#player)
      * [`[parameters]` - A list of parameters to pass to the starting movie](#parameters---a-list-of-parameters-to-pass-to-the-starting-movie)
      * [`script_timeout` - Script execution timeout](#script_timeout---script-execution-timeout)
      * [`base_url` - Base URL for relative file paths](#base_url---base-url-for-relative-file-paths)
      * [`quality` - Quality that the content starts with](#quality---quality-that-the-content-starts-with)
      * [`align` - Stage Alignment that the content starts with](#align---stage-alignment-that-the-content-starts-with)
      * [`force_align` - Allow or disallow the content from changing its Stage Alignment](#force_align---allow-or-disallow-the-content-from-changing-its-stage-alignment)
      * [`scale_mode` - Stage Scale Mode that the content starts with](#scale_mode---stage-scale-mode-that-the-content-starts-with)
      * [`force_scale_mode` - Allow or disallow the content from changing its Stage Alignment](#force_scale_mode---allow-or-disallow-the-content-from-changing-its-stage-alignment)
      * [`upgrade_http_to_https` - Whether to upgrade HTTP urls to HTTPS silently](#upgrade_http_to_https---whether-to-upgrade-http-urls-to-https-silently)
      * [`load_behavior` - How Ruffle should load movies](#load_behavior---how-ruffle-should-load-movies)
      * [`letterbox` - Controls visual letterboxing around the content](#letterbox---controls-visual-letterboxing-around-the-content)
      * [`spoof_url` - URL to pretend the initial SWF is being loaded from](#spoof_url---url-to-pretend-the-initial-swf-is-being-loaded-from)
      * [`version` - Version of the Flash Player to emulate](#version---version-of-the-flash-player-to-emulate)
      * [`runtime` - Which type of runtime to emulate](#runtime---which-type-of-runtime-to-emulate)
      * [`frame_rate` - Override the target frame rate of this movie](#frame_rate---override-the-target-frame-rate-of-this-movie)
      * [`mock_external_interface` - Provide a mocked ExternalInterface](#mock_external_interface---provide-a-mocked-externalinterface)
<!-- TOC -->

## Directory structure

- `ruffle-bundle.toml` - **required**, the bundle information
- `content/` - a directory containing any swf files, assets they need, etc.

More files and folders may be added in the future, as this format is expanded upon.

### `ruffle-bundle.toml` (Bundle information)
This [toml](https://toml.io/) file is required and contains information that Ruffle needs to run this bundle.

See [the ruffle-bundle.toml file specification](#ruffle-bundletoml-file-specification) for more details.

### `content/` (Flash content)
Every file (and subdirectory) within this directory will be accessible to the Flash content, exposed as a **virtual filesystem**.

To Flash content, this is accessible through `file:///` - for example, the file `/content/game.swf` is `file:///game.swf`.
The file `/content/locale/en.xml` is `file:///locale/en.xml`.

You'll want to put the `.swf` file in here, along with any extra files it may need. Files outside this directory are **not** accessible to the content.

## `ruffle-bundle.toml` file specification
The absolute minimum `ruffle-bundle.toml` looks like this:
```toml
[bundle]
name = "Super Mario 63"
url = "file:///game.swf"
```

If either `bundle.name` or `bundle.url` is invalid or missing, the bundle is considered to be invalid and will not open.
The same is true if the toml document is malformed or corrupt.

All other fields are absolutely optional and reasonable defaults will be assumed if they're missing or invalid.

### `[bundle]`
This section is required to exist, and contains the two required fields for a bundle to work in Ruffle:

#### `name` - The name of the bundle
This can be anything, and is shown to the user in UI.
Try to keep this a reasonable length, and descriptive about what this bundle actually *is*.

#### `url` - The url of the Flash content to open
Whilst this **can** be a URL on the internet (for example, `url = "https://ruffle.rs/demo/logo-anim.swf"` is totally valid),
we would recommend that it instead be the path to a file within the `content/` directory inside the bundle.

This way, an internet connection is not required, and the bundle won't stop working in 5 years when the website changes.

Remember - the `content/` directory is accessible through `file:///` - so if you have a game at `content/game.swf`, you'll want to use `url = "file:///game.swf"`.


### `[player]`
This section contains player options, which change how Ruffle emulates the content in this bundle.

These options may be overridden by users.

#### `[parameters]` - A list of parameters to pass to the starting movie
These are sometimes also called 'FlashVars' by Flash developers. This is appended to any parameters given in the query string of the `bundle.url`.

All values must be strings.

Example:
```toml
[player.parameters]
key = "value"
favourite_number = "5"
```

#### `script_timeout` - Script execution timeout
How long a single script execution (e.g. a frame of ActionScript 3) can take, before it's considered to be stuck or broken.
Value should be in seconds - fractional values are allowed.

Example with a 5-second limit on scripts:
```toml
[player]
script_timeout = 5
```

#### `base_url` - Base URL for relative file paths
By default, this is the `bundle.url`, but some content may require something else.

When content looks up a **relative** path, as opposed to an absolute path, it makes it relative to this base url instead.

Example:
```toml
[player]
base_url = "https://example.org"
```

#### `quality` - Quality that the content starts with
The movie has the capacity to change this automatically at runtime, and the user may also change it at will.
The default value generally depends on the users hardware, and it's advisable to leave it to do so unless content requires
a specific quality for aesthetics.

Whilst Flash [does technically support many quality modes](https://web.archive.org/web/20240420201659/https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/StageQuality.html);
Ruffle currently only implements `low`, `medium` and `high`.

Example:
```toml
[player]
quality = "low"
```

#### `align` - Stage Alignment that the content starts with
The movie has the capacity to change this automatically at runtime, unless `player.force_align` is set to `true`.

This controls the position of the movie after scaling to fill the viewport.

This may be one of the following values:
- `bottom`: Specifies that the Stage is aligned at the bottom.
- `bottom_left`: Specifies that the Stage is aligned in the bottom-left corner.
- `bottom_right`: Specifies that the Stage is aligned in the bottom-right corner.
- `left`: Specifies that the Stage is aligned on the left.
- `right`: Specifies that the Stage is aligned to the right.
- `top`: Specifies that the Stage is aligned at the top.
- `top_left`: Specifies that the Stage is aligned in the top-left corner.
- `top_right`: Specifies that the Stage is aligned in the top-right corner.
- `center` (Default): Specified that the Stage is aligned in the center.

Example:
```toml
[player]
align = "bottom_right"
```

#### `force_align` - Allow or disallow the content from changing its Stage Alignment
If set to `true`, content may not change its own Stage Alignment value (see `player.align`). Default is `false`.

#### `scale_mode` - Stage Scale Mode that the content starts with
The movie has the capacity to change this automatically at runtime, unless `player.force_scale_mode` is set to `true`.

This controls the behavior when the player viewport size differs from the SWF size.

This may be one of the following values:
- `exact_fit`: The movie will be stretched to fit the container.
- `no_border`: The movie will maintain its aspect ratio, but will be cropped.
- `no_scale`: The movie is not scaled to fit the container, and the content is assumed to adjust itself with scripts.
- `show_all` (Default): The movie will scale to fill the container and maintain its aspect ratio, but will be letterboxed.

Example:
```toml
[player]
scale_mode = "no_border"
```

#### `force_scale_mode` - Allow or disallow the content from changing its Stage Alignment
If set to `true`, content may not change its own Stage Scale Mode value (see `player.scale_mode`). Default is `false`.

#### `upgrade_http_to_https` - Whether to upgrade HTTP urls to HTTPS silently
If `true`, all `http://` URLs will be replaced with `https://`.

#### `load_behavior` - How Ruffle should load movies
Some movies expect to be streamed, or expect to load instantly. This allows you to work around any potential issues by
forcing a specific loading behaviour.

This may be one of the following values:
- `streaming` (Default): Allow movies to execute before they have finished loading.
- `delayed`: Delay execution of loaded movies until they have finished loading.
- `blocking`: Block Ruffle until movies have finished loading.

#### `letterbox` - Controls visual letterboxing around the content
If the contents aspect ratio does not match the players aspect ratio, Ruffle may put up letterboxes for aesthetics and
to hide objects that perhaps should not be visible.

This may be one of the following values:
- `off`: The content will not be letterboxed, everything will be visible.
- `fullscreen`: The content will only be letterboxed if viewed in Full Screen and the aspect ratio doesn't match.
- `on` (Default): The content will always be letterboxed if the aspect ratio doesn't match.

#### `spoof_url` - URL to pretend the initial SWF is being loaded from
If set to a valid URL, we will **pretend** that the `bundle.url` SWF is actually being loaded from this location instead.
This is often required for site locks that check if the content is being loaded from the original domain.

We do not *actually* load the URL, and all other assets/SWFs are not affected.

#### `version` - Version of the Flash Player to emulate
Whilst it's not common, some content depends on behaviour from specific Flash Players. You may set this to force Ruffle
to try and emulate that behaviour.

Default is likely to be `32`, but may be subject to change.

#### `runtime` - Which type of runtime to emulate
Unfortunately content does not indicate which runtime it wants, so if there's AIR content it must be called out by setting
this value appropriately.

This may be one of the following values:
- `flash_player` (Default): The original Flash Player, no thrills.
- `air`: The Adobe AIR runtime, an extension of Flash Player with more native capabilities.

#### `frame_rate` - Override the target frame rate of this movie
If set, the movies preferred frame rate is ignored and this value is used instead.
May be used to speed up or slow down movies, but some content keeps its own time tracking and may not be affected. Value can both be an integer and a fractional value.

Example:
```toml
[player]
frame_rate = 30.0
```

#### `mock_external_interface` - Provide a mocked ExternalInterface
Some content used JavaScript calls to query things like the page URL. By setting this value to `true`, Ruffle will provide
a mocked up ExternalInterface that responds to some of the common JavaScript calls appropriately.
