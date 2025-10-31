package ruffle.steamworks {
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.external.ExternalInterface;

    import ruffle.steamworks.Achievement;
    import ruffle.steamworks.LocalPlayer;
    import ruffle.steamworks.Utils;

    public class Client extends EventDispatcher {
        public static var USER_STATS_RECEIVED: String = "userStatsReceived";

        public function Client(appid: *) {
            var error: * = appid ? call("client.init", appid)
                                 : call("client.init");
            if (error) {
                throw new Error(error);
            }

            var self: Client = this;
            ExternalInterface.addCallback("steamworks.callbacks.userStatsReceived", function (): void {
                self.dispatchEvent(new Event(Client.USER_STATS_RECEIVED));
            });
        }

        public function get achievement(): Achievement {
            if (!_achievement) {
                _achievement = new Achievement(this);
            }
            return _achievement;
        }

        public function get localPlayer(): LocalPlayer {
            if (!_localPlayer) {
                _localPlayer = new LocalPlayer(this);
            }
            return _localPlayer;
        }

        public function get utils(): Utils {
            if (!_utils) {
                _utils = new Utils(this);
            }
            return _utils;
        }

        internal function call(name: String, ...args: *): * {
            return ExternalInterface.call.apply(null, ["steamworks." + name].concat(args));
        }

        private var _achievement: Achievement;
        private var _localPlayer: LocalPlayer;
        private var _utils: Utils;
    }
}
