package {
	import flash.net.NetConnection;

	public class Test {
		public function Test() {
			var connection = new NetConnection();
			connection.addEventListener("netStatus", function(e) {
				trace("Event: " + e);
				trace("Code: " + e.info.code);
				trace("Level: " + e.info.level);
			});
			trace("//isConnected:", connection.connected);
			trace("Calling connect");
			connection.connect(null);
			trace("Called connect");
			trace("//isConnected:", connection.connected);
		}
	}
}