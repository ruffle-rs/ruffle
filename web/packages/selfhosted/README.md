# ruffle-selfhosted

ruffle-selfhosted is the intended way to get Ruffle onto your website.

You may either include it and forget about it, and we will polyfill existing Flash content,
or use our APIs for custom configurations or more advanced usages of the Ruffle player.

## Using ruffle-selfhosted

For more examples and in-depth documentation on how to use Ruffle on your website, please
[check out our wiki](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#web).

### Host Ruffle

Before you can get started with using Ruffle on your website, you must host its files yourself.
Either take the [latest build](https://ruffle-rs.s3-us-west-1.amazonaws.com/builds/web/ruffle_web_latest.zip)
or [build it yourself](../../README.md), and make these files accessible by your web server.

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
        let player = ruffle.create_player();
        let container = document.getElementById("container");
        container.appendChild(player);
        player.stream_swf_url("movie.swf");
    });
</script>
<script src="path/to/ruffle/ruffle.js"></script>
```

## Building, testing or contributing

Please see [the ruffle-web README](../../README.md).
