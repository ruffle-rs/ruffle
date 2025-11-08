package ruffle.steamworks {
    import ruffle.steamworks.Client;

    public class Utils {
        private var _client: Client;
        public function Utils(client: Client) {
            _client = client;
        }

        public function getAppId(): uint {
            return _client.call("utils.getAppId");
        }
        public function isSteamRunningOnSteamDeck(): Boolean {
            return _client.call("utils.isSteamRunningOnSteamDeck");
        }
    }
}
