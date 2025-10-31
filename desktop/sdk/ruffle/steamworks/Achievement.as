package ruffle.steamworks {
    import ruffle.steamworks.Client;

    public class Achievement {
        private var _client: Client;
        public function Achievement(client: Client) {
            _client = client;
        }

        public function activate(name: String): Boolean {
            return _client.call("achievement.activate", name);
        }
        public function isActivated(name: String): Boolean {
            return _client.call("achievement.isActivated", name);
        }
        public function clear(name: String): Boolean {
            return _client.call("achievement.clear", name);
        }
        public function names(): Array {
            return _client.call("achievement.names");
        }
    }
}
