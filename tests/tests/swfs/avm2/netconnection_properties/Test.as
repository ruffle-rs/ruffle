package  {
	
	import flash.display.MovieClip;
import flash.net.NetConnection;
import flash.net.NetConnection;
	import flash.events.NetStatusEvent;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var connection:NetConnection = new NetConnection();

			traceProperties(connection);
			trace("");

			trace("/// connection.connect(null)");
			connection.connect(null);
			traceProperties(connection);
			trace("");

			trace("/// connection.connect(\"http://example.org\")");
			connection.connect("http://example.org");
			traceProperties(connection);
			trace("");
			
			trace("/// connection.close()");
			connection.close();
			traceProperties(connection);
			trace("");

			trace("/// connection.connect(\"https://example.org\")");
			connection.connect("https://example.org");
			traceProperties(connection);
			trace("");
		}

		function traceSafe(connection: NetConnection, key: String) {
			try {
				var value = connection[key];
				if (typeof(value) === "string") {
					trace("connection." + key + " = \"" + escapeString(value) + "\"");
				} else if (value is Array) {
					trace("connection." + key + " = [" + value + "]");
				} else if (value === connection) {
					trace("connection." + key + " = connection");
				} else {
					trace("connection." + key + " = " + value);
				}
			} catch (error) {
				trace("connection." + key + " = " + error);
			}
		}

		function traceProperties(connection: NetConnection) {
			traceSafe(connection, "client");
			traceSafe(connection, "connected");
			traceSafe(connection, "connectedProxyType");
			traceSafe(connection, "farID");
			traceSafe(connection, "farNonce");
			traceSafe(connection, "maxPeerConnections");
			traceSafe(connection, "nearID");
			traceSafe(connection, "nearNonce");
			traceSafe(connection, "objectEncoding");
			traceSafe(connection, "protocol");
			traceSafe(connection, "proxyType");
			traceSafe(connection, "unconnectedPeerStreams");
			traceSafe(connection, "uri");
			traceSafe(connection, "usingTLS");
		}

		function escapeString(input: String): String {
			var output:String = "";
			for (var i:int = 0; i < input.length; i++) {
				var char:String = input.charAt(i);
				switch (char) {
					case "\\":
						output += "\\\\";
						break;
					case "\"":
						output += "\\\"";
						break;
					case "\n":
						output += "\\n";
						break;
					case "\r":
						output += "\\r";
						break;
					case "\t":
						output += "\\t";
						break;
					default:
						output += char;
				}
			}
			return output;
		}
	}
}
