Welcome to our ActionScript SDK used to extend ActionScript 3 with new native functionality when using the Ruffle desktop build.

# Steamworks

Currently this SDK includes support for the [Steamworks™ API](https://partner.steamgames.com/doc/api) [^1].
Internally Ruffle uses the [steamworks](https://github.com/Noxime/steamworks-rs) Rust crate as wrapper around the API.
See also their README, especially the section on "Help, I can't run my game!".
To expose this functionality to AS3 code, the API is exposed as an `ExternalInterface`.
Note: This means the ruffle option `--dummy-external-interface` is incompatible with using the steamworks API.

## Building Ruffle

When building ruffle you have to make sure to enable the `steamworks` feature, e.g. like this `cargo build --release --feature steamworks`.

## Usage

Include the SDK, e.g. with `mxmlc -compiler.source-path=/path/to/ruffle/desktop/sdk/`. The API is heavily inspired by [steamworks.js](https://github.com/ceifa/steamworks.js/).

Code example:
```
package {
    import ruffle.steamworks.Client;

    public class Test {
        function Test() {
            var client: Client = new Client(/* appId */ 480); // Or new Client() when using steam_appid.txt

            trace("appId: " + client.utils.getAppId());

            trace("name: " + client.localPlayer.getName());
            trace("level: " + client.localPlayer.getLevel());
            trace("getIpCountry: " + client.localPlayer.getIpCountry());

            client.addEventListener(Client.USER_STATS_RECEIVED, function(): void {
                trace("Event: USER_STATS_RECEIVED");
                trace("isActived('ACH_WIN_ONE_GAME'): " + client.achievement.isActivated('ACH_WIN_ONE_GAME'));

                trace(client.achievement.names());
            });

            client.localPlayer.requestUserStats();
        }
    }
}
```

[^1]: Steamworks is a trademark of the Valve Corporation.


