package  {
	
	import flash.display.MovieClip;
	import flash.net.LocalConnection;
	import flash.utils.getQualifiedClassName;
	
	
	public class Child extends MovieClip {
		var lc: LocalConnection = new LocalConnection();
		
		public function Child() {
			lc.connect("avm2_child");
			lc.client = {};
			lc.client.test = function() {
				trace("avm2_child.test was called with " + arguments.length + " argument" + (arguments.length == 0 ? "" : "s"));
				if (arguments.length > 0) {
					trace("  " + repr(arguments));
				}
			}
		}
		
		private function getObjectId(needle: Object, haystack: Array): String {
			for (var i = 0; i < haystack.length; i++) {
				if (haystack[i] === needle) {
					return i;
				}
			}
			return null;
		}
		
		public function repr(value: *, indent: String = "  ", seenObjects: Array = null) {
			if (seenObjects == null) {
				seenObjects = [];
			}
			if (value === lc) {
				return "lc";
			}
			
			if (value === undefined || value === null || value === true || value === false || value is Number) {
				return String(value);
			} else if (value is String) {
				return escapeString(value);
			} else {
				var existingId = getObjectId(value, seenObjects);
				if (existingId != null) {
					return "*" + existingId;
				}
				existingId = seenObjects.length;
				seenObjects.push(value);
				if (value is Array) {
					if (value.length == 0) {
						return "*" + existingId + " []";
					} else {
						var result = "*" + existingId + " [\n";
						var nextIndent = indent + "  ";
						for (var i = 0; i < value.length; i++) {
							result += nextIndent + repr(value[i], nextIndent, seenObjects) + "\n";
						}
						return result + indent + "]";
					}
				} else {
					var keys = [];
					for (var key in value) {
						keys.push(key);
					}
					keys.sort();
			
					var result = "*" + existingId + " " + getQualifiedClassName(value) + " {";
			
					if (keys.length == 0) {
						return result + "}";
					} else {
						result += "\n";
						var nextIndent = indent + "  ";
						for (var i = 0; i < keys.length; i++) {
							result += nextIndent + keys[i] + " = " + repr(value[keys[i]], nextIndent, seenObjects) + "\n";
						}
						return result + indent + "}";
					}
				}
			}
		}
		
		public function escapeString(input: String): String {
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
