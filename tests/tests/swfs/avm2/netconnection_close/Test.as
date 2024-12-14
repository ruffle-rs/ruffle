package  {
	
	import flash.display.MovieClip;
	import flash.net.NetConnection;
	import flash.events.NetStatusEvent;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var connection:NetConnection = new NetConnection();
			connection.addEventListener(NetStatusEvent.NET_STATUS, onStatusChange);
			
			trace("/// connection.close()");
			connection.close();
			trace("");
			
			trace("/// connection.connect(null)");
			connection.connect(null);
			trace("");
			
			trace("/// connection.connect(null)");
			connection.connect(null);
			trace("");
			
			trace("/// connection.close()");
			connection.close();
			trace("");
			
			trace("/// connection.connect(\"http://example.org\")");
			connection.connect("http://example.org");
			trace("");
			
			trace("/// connection.close()");
			connection.close();
			trace("");
		}
		
		function onStatusChange(event: NetStatusEvent) {
			trace("-- NetStatusEvent.NET_STATUS event start --");
			trace("event.type = " + event.type);
			trace("event.bubbles = " + event.bubbles);
			trace("event.cancelable = " + event.cancelable);
			
			var keys = [];
			for (var key in event.info) {
				keys.push(key);
			}
			keys.sort();
			for each (var key in keys) {
				trace("event.info." + key + " = " + event.info[key] + " (" + typeof(event.info[key]) + ")");
			}
			
			trace("-- NetStatusEvent.NET_STATUS event end --");
		}
	}
	
}
