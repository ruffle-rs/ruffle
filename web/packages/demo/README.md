# ruffle-demo

ruffle-demo is an example of how to include Ruffle in your website.
It also serves as a nice local test to run Ruffle in the web locally, for developers.

## Using ruffle-demo

### Hosted demo

To view this demo online right now, [check out the hosted demo](https://ruffle.rs/demo).

It's exactly the same code as this directory, updated nightly.

### Run your own demo

After [building ruffle-web](https://github.com/ruffle-rs/ruffle/blob/master/web/README.md#building-from-source),
you can run `npm run demo` in the `web` folder to launch the demo.

It will start a local web server and print the address in the console.
Navigate to that website (usually [http://localhost:8080](http://localhost:8080)) in your browser.

### Configuring the demo

The demo provides the ability to have a list of sample SWFs to choose from.
This can be helpful if you have a list of useful SWFs to test through, and we use it ourselves
to showcase Ruffle on various games or animations.

To use this, add a new file `swfs.json` in this directory. The contents should look like this:

```json
{
    "swfs": [
        {
            "location": "swfs/alien_hominid.swf",
            "title": "Alien Hominid",
            "author": "Tom Fulp and Dan Paladin",
            "authorLink": "https://www.newgrounds.com",
            "type": "Game"
        },
        {
            "location": "swfs/saturday_morning_watchmen.swf",
            "title": "Saturday Morning Watchmen",
            "author": "Harry Partridge",
            "authorLink": "https://twitter.com/HappyHarryToons",
            "type": "Animation"
        },
        {
            "location": "swfs/synj1.swf",
            "title": "Synj vs. Horrid Part 1",
            "author": "Dan Paladin",
            "authorLink": "https://www.thebehemoth.com",
            "type": "Animation"
        },
        {
            "location": "swfs/synj2.swf",
            "title": "Synj vs. Horrid Part 2",
            "author": "Dan Paladin",
            "authorLink": "https://www.thebehemoth.com",
            "type": "Animation"
        },
        {
            "location": "swfs/wasted_sky.swf",
            "title": "Wasted Sky",
            "author": "Tom Fulp",
            "authorLink": "https://www.newgrounds.com",
            "type": "Game"
        }
    ]
}
```

## Building, testing or contributing

Please see [the ruffle-web README](https://github.com/ruffle-rs/ruffle/blob/master/web/README.md).
