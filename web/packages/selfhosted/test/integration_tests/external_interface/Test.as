package  {
	import flash.display.MovieClip;
	import flash.external.ExternalInterface;
	import flash.utils.getQualifiedClassName;
	import flash.utils.setTimeout;
	
	
	public class Test extends MovieClip {
		public function Test() {
			log("ExternalInterface.available: " + repr(ExternalInterface.available));
			log("ExternalInterface.objectID: " + repr(ExternalInterface.objectID));
			
			try {
				ExternalInterface.addCallback("callMethodImmediately", function(name: String) {
					log("callMethodImmediately called with " + arguments.length + " argument" + (arguments.length == 1 ? "" : "s"));
					log("  " + repr(arguments, "  "));
					try {
						log("  call(" + name + ", ...) = " + repr(ExternalInterface.call.apply(null, arguments)));
					} catch (e) {
						log("  call(" + name + ", ...) = " + e);
					}
				});
				ExternalInterface.addCallback("callMethodWithDelay", function(name: String) {
					log("callMethodWithDelay called with " + arguments.length + " argument" + (arguments.length == 1 ? "" : "s"));
					log("  " + repr(arguments, "  "));
					var args = arguments;
					setTimeout(function() {
						try {
							log("  call(" + name + ", ...) = " + repr(ExternalInterface.call.apply(null, args)));
						} catch (e) {
							log("  call(" + name + ", ...) = " + e);
						}
					}, 1);
				});
				ExternalInterface.addCallback("log", function() {
					log("log called with " + arguments.length + " argument" + (arguments.length == 1 ? "" : "s"));
					log("  " + repr(arguments, "  "));
				});
				ExternalInterface.addCallback("returnAValue", function(value: *) {
					log("returnAValue called with " + repr(value));
					log("  " + repr(arguments, "  "));
					return value;
				});
				ExternalInterface.addCallback("throwAnException", function() {
					log("throwAnException called");
					throw new ArgumentError("Custom Argument Error!", 123);
				});
				ExternalInterface.addCallback("setMarshallExceptions", function(value: Boolean) {
					log("setMarshallExceptions called with " + repr(value));
					ExternalInterface.marshallExceptions = value;
				});
				ExternalInterface.addCallback("addAnotherCallback", function(name: String, returnValue: *) {
					log("addAnotherCallback called for " + repr(name) + " to return " + repr(returnValue));
					ExternalInterface.addCallback(name, function() {
					log(name + " called");
						return returnValue;
					});
				});
			} catch (e) {
				log("Error adding callbacks: " + e);
			}
		}

		function log(value: *) {
			trace(value);
			result.text += value + "\n";
		}
		
		function repr(value: *, indent: String = "  ") {
			if (value === undefined || value === null || value === true || value === false || value is Number) {
				return value;
			} else if (value is String) {
				return escapeString(value);
			} else if (value is Array) {
				if (value.length == 0) {
					return "[]";
				} else {
					var result = "[\n";
					var nextIndent = indent + "  ";
					for (var i = 0; i < value.length; i++) {
						result += nextIndent + repr(value[i], indent + nextIndent) + "\n";
					}
					return result + indent + "]";
				}
			} else {
				var keys = [];
				for (var key in value) {
					keys.push(key);
				}
				keys.sort();
				
				var result = getQualifiedClassName(value) + " {";
				
				if (keys.length == 0) {
					return result + "}";
				} else {
					result += "\n";
					var nextIndent = indent + "  ";
					for (var i = 0; i < keys.length; i++) {
						result += nextIndent + keys[i] + " = " + repr(value[keys[i]], nextIndent) + "\n";
					}
					return result + indent + "}";
				}
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
