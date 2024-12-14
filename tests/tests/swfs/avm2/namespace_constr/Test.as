package {
	public class Test {
		function Test() {
			namespace example = "value";

			trace("namespace example = \"value\"");
			dump(example);
			trace("");

			var otherNS = new Namespace("otherPrefix", "otherUri");
			var qName = new QName("namespace", "name");
			var values = [null, undefined, "test", "", "NOT A VALID PREFIX", otherNS, qName];
			
			trace("new Namespace()");
			try {
				dump(new Namespace());
			} catch (ex) {
				trace("! " + ex);
			}
			trace();

			for (var i = 0; i < values.length; i++) {
				test1(values[i]);
			}

			for (var i = 0; i < values.length; i++) {
				for (var j = 0; j < values.length; j++) {
					test2(values[i], values[j]);
				}
			}
			
			function test1(a) {
				trace("new Namespace(" + repr(a) + ")");
				try {
					dump(new Namespace(a));
				} catch (ex) {
					trace("! " + ex);
				}
				trace();
				
				trace("Namespace(" + repr(a) + ")");
				try {
					dump(Namespace(a));
				} catch (ex) {
					trace("! " + ex);
				}
				trace();
			}
			
			function test2(a, b) {
				trace("new Namespace(" + repr(a) + ", " + repr(b) + ")");
				try {
					dump(new Namespace(a, b));
				} catch (ex) {
					trace("! " + ex);
				}
				trace();
			}
			
			
			function dump(ns: Namespace) {
				trace(" prefix: " + repr(ns.prefix));
				trace(" uri: " + repr(ns.uri));
			}
			
			function repr(value: *) {
				if (value === undefined) {
					return "undefined";
				} else if (value === null) {
					return "null";
				} else if (value === otherNS) {
					return "otherNS";
				} else if (value === qName) {
					return "qName";
				} else if (value is String) {
					return escapeString(value);
				} else {
					return typeof(value) + " " + value;
				}
			}
			
			function escapeString(input: String): String {
				var output:String = "\"";
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
				return output + "\"";
			}
		}
	}
}



