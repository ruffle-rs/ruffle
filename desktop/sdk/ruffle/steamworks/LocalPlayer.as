package ruffle.steamworks {
    import ruffle.steamworks.Client;

    public class LocalPlayer {
        private var _client: Client;
        public function LocalPlayer(client: Client) {
            _client = client;
        }

        public function getSteamId(): String {
            return _client.call("localplayer.getSteamId");
        }
        public function getName(): String {
            return _client.call("localplayer.getName");
        }
        public function getLevel(): uint {
            return _client.call("localplayer.getLevel");
        }
        public function getIpCountry(): String {
            return _client.call("localplayer.getIpCountry");
        }
        public function requestUserStats(): void {
            _client.call("localplayer.requestUserStats");
        }
    }
}
