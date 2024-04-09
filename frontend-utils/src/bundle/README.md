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

