# ruffle-core

ruffle-core is the core javascript bindings to the Wasm ruffle-web binary,
and contains the actual public API.

## Using ruffle-core

For more examples and in-depth documentation on how to use Ruffle on your website, please
[check out our wiki](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#web).

### Host Ruffle

The `core` package is configured for websites that build and bundle their files themselves.
Simply add `ruffle` to an npm package, and use something like Webpack to actually bundle and serve
the files.

If you wish to use Ruffle on a website that doesn't use npm, we have a pre-bundled version which
we call 'selfhosted'. Please [refer to its documentation](https://github.com/ruffle-rs/ruffle/tree/master/web/packages/selfhosted).

Please note that the `.wasm` file must be served properly, and some web servers may not do that
correctly out of the box. Please see [our wiki](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-wasm-mime-type)
for instructions on how to configure this, if you encounter a `Incorrect response MIME type` error.

### "Plug and Play"

If you have an existing website with flash content, you can simply include Ruffle as a script and
our polyfill magic will replace everything for you. No fuss, no mess.

```html
<script src="path/to/ruffle/ruffle.js"></script>
```

### Javascript API

If you want to control the Ruffle player, you may use our Javascript API.

```html
<script>
    window.RufflePlayer = window.RufflePlayer || {};

    window.addEventListener("DOMContentLoaded", () => {
        let ruffle = window.RufflePlayer.newest();
        let player = ruffle.createPlayer();
        let container = document.getElementById("container");
        container.appendChild(player);
        player.ruffle().load("movie.swf");
    });
</script>
<script src="path/to/ruffle/ruffle.js"></script>
```

## Building, testing or contributing

Please see [the ruffle-web README](../../README.md).
