package  {
	
	import flash.display.MovieClip;
	import flash.net.LocalConnection;
	import flash.utils.getQualifiedClassName;
	import flash.events.AsyncErrorEvent;
	import flash.events.Event;
	import flash.events.SecurityErrorEvent;
	import flash.events.StatusEvent;
	import flash.system.fscommand;
	import flash.net.URLRequest;
	import flash.display.Loader;
	
	
	public class Test extends MovieClip {
		// Just a safe value, to make things easier...
		// Sometimes Flash isn't super consistant about things being done on the same frame
		const TICKS_PER_TEST: uint = 3;

		var receiver: LocalConnection = new LocalConnection();
		var sender: LocalConnection = new LocalConnection();
		var custom: LocalConnection;
		var recvObject: Object = {};
		var tests: Array = [];
		var currentTest = null;
		var frameNum: uint = 0;
		var totalFrameNum: uint = 0;
		var funkyNames = [null, 0, "", " ??? "];
		var protectedFunctions = ["send", "connect", "close", "allowDomain", "allowInsecureDomain", "domain"];
		
		public function Test() {
			custom = new CustomLocalConnection(this);

			loadMovie("avm2child/child.swf");
			loadMovie("avm1child/child.swf");
			
			trace("LocalConnection.isSupported: " + repr(LocalConnection.isSupported));
			trace("");
			
			recvObject.test = createTestFunction("recvObject.test");
			
			setupEvents(receiver);
			setupEvents(sender);
			setupEvents(custom);
			
			addTest("A message to nowhere!", function() {
				send(sender, "nowhere", "test");
			});
			
			addTest("Both receivers try to connect to the same channel", function() {
				connect(receiver, "channel");
				connect(custom, "channel");
			});
			
			addTest("A message to an unimplemented function", function() {
				send(sender, "channel", "unimplemented");
			});
			
			addTest("Receiver tries to connect elsewhere, but can't", function() {
				connect(receiver, "elsewhere");
			});
			
			addTest("Receiver actually connects elsewhere, and custom is allowed to connect to channel", function() {
				close(receiver);
				connect(receiver, "elsewhere");
				connect(custom, "channel");
			});
			
			addTest("Sender calls test() on 'channel'", function() {
				send(sender, "channel", "test");
			});
			
			addTest("Client is used", function() {
				receiver.client = recvObject;
				send(sender, "elsewhere", "test");
			});
			
			addTest("Sender calls test() on 'channel'... after the listener is gone", function() {
				close(custom);
				send(sender, "channel", "test");
			});
			
			addTest("Sender calls test() on 'elsewhere'... immediately before the listener is gone", function() {
				send(sender, "elsewhere", "test");
				close(receiver);
			});
			
			addTest("Sender calls test() on 'channel'... before the listener connects", function() {
				send(sender, "channel", "test");
				connect(custom, "channel");
			});
			
			addTest("Sending to a channel that gets reassigned before end-of-frame", function() {
				send(sender, "channel", "test");
				close(custom);
				connect(receiver, "channel");
			});
			
			addTest("Channels reconnect and receive", function() {
				close(custom);
				close(receiver);
				connect(receiver, "elsewhere");
				send(sender, "channel", "test");
				send(sender, "elsewhere", "test");
				connect(custom, "channel");
			});
			
			addTest("A connected listener can also send", function() {
				send(receiver, "channel", "test");
				send(receiver, "elsewhere", "test");
			});
			
			addTest("A listener throws an error", function() {
				// Fun fact: you can crash Flash Player if the thing thrown isn't an object
				send(sender, "channel", "throwAnError");
			});
			
			addTest("Close something's that's already closed", function() {
				sender.close();
			});
			
			addTest("Send to funky channel names", function() {
				for (var i = 0; i < funkyNames.length; i++) {
					send(sender, funkyNames[i], "test");
				}
			});
			
			addTest("Send to funky methods", function() {
				for (var i = 0; i < funkyNames.length; i++) {
					send(sender, "channel", funkyNames[i]);
				}
			});
			
			addTest("Connect to funky names", function() {
				for (var i = 0; i < funkyNames.length; i++) {
					connect(sender, funkyNames[i]);
					close(sender);
				}
			});
			
			addTest("Connect to something with a prefix", function() {
				connect(sender, "localhost:something");
				close(sender);
			});
			
			addTest("Send to protected methods", function() {
				for (var i = 0; i < protectedFunctions.length; i++) {
					send(sender, "channel", protectedFunctions[i]);
				}
			});
			
			addTest("Arguments are sent", function() {
				send(sender, "elsewhere", "test", 1, "two", {value: 3}, [4, 5]);
			});
			
			addTest("Explicit host prefix", function() {
				send(sender, "localhost:channel", "test");
				send(sender, "notlocalhost:elsewhere", "test");
			});
			
			addTest("Underscores in names", function() {
				close(custom);
				connect(custom, "_channel");
				send(sender, "_channel", "test");
			});
			
			addTest("Underscores in name doesn't allow a prefix", function() {
				send(sender, "localhost:channel", "test");
				send(sender, "localhost:_channel", "test");
			});
			
			addTest("Case sensitivity", function() {
				send(sender, "ELSEWhere", "test");
				send(sender, "LOCalHOST:ElseWhere", "test");
			});
			
			addTest("Calling an AVM2 movie", function() {
				send(sender, "avm2_child", "test");
			});
			
			addTest("Calling an AVM1 movie", function() {
				send(sender, "avm1_child", "test");
			});
			
			addTest("Argument translations: primitives", function() {
				sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", 1, 1.2, true, false, "string", null, undefined);
			});
			
			addTest("Argument translations: simple array", function() {
				sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", [1,2, "three", 4.5, NaN, Infinity]);
			});
			
			addTest("Argument translations: simple object", function() {
				sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", {"nested": {"numbers": [1,2], "string": "hello"}});
			});
			
			// [NA] broken in ruffle at time of writing
			//addTest("Argument translations: self referential object", function() {
			//	var obj = {};
			//	obj.self = obj;
			//	obj.nested = {root: obj};
			//	sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", obj);
			//});
			
			//addTest("Argument translations: self referential array", function() {
			//	var array = [];
			//	array.push(array);
			//	sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", array);
			//});
			
			//addTest("Argument translations: vector", function() {
				//var vector = new Vector.<String>();
				//vector.push("hello");
				//vector.push("world");
				//sendToMany(sender, ["avm1_child", "avm2_child", "_channel"], "test", vector);
			//});

			addTest("AVM1 movie throws an error", function() {
				send(sender, "avm1_child", "throwAnError");
			});
			
			addEventListener(Event.ENTER_FRAME, onEnterFrame);
		}

		function loadMovie(path: String) {
			var loader = new Loader();
			loader.load(new URLRequest(path));
			addChild(loader);
		}
		
		function onEnterFrame(event: Event) {
			totalFrameNum++;

			if (frameNum == TICKS_PER_TEST) {
				trace("");
				trace("-- end test: " + currentTest[0] + " --");
				trace("");
				frameNum = 0;
				return; // Allow any end-of-frame cleanup before next test
			}
			if (frameNum == 0) {
				currentTest = tests.shift();
				if (currentTest != null) {
					trace("");
					trace("-- start test: " + currentTest[0] + " --");
					trace("");
					try {
						currentTest[1]();
					} catch (e) {
						trace("! test stopped with error: " + e);
					}
					trace("");
					trace("-- end frame: " + currentTest[0] + " --");
					trace("");
				}
			}
			if (currentTest == null) {
				trace("Finished after " + totalFrameNum + " frames");
				fscommand("exit");
				removeEventListener(Event.ENTER_FRAME, onEnterFrame);
				return;
			}
			frameNum++;
		}
		
		function connect(lc: LocalConnection, name: String) {
			var doing = repr(lc) + ".connect(" + repr(name) + ")";
			try {
				lc.connect(name);
				trace(doing);
			} catch (e) {
				trace(doing + ": ! " + e);
			}
		}
		
		function send(lc: LocalConnection, connectionName: String, methodName: String, ...args) {
			var doing = repr(lc) + ".send(" + repr(connectionName) + ", " + repr(methodName) + ", " + repr(args) + ")";
			try {
				args.unshift(methodName);
				args.unshift(connectionName);
				lc.send.apply(lc, args);
				trace(doing);
			} catch (e) {
				trace(doing + ": ! " + e);
			}
		}
		
		function sendToMany(lc: LocalConnection, connectionNames: Array, methodName: String, ...args) {
			args.unshift(methodName);
			args.unshift("");
			args.unshift(lc);
			for (var i = 0; i < connectionNames.length; i++) {
				args[1] = connectionNames[i];
				send.apply(null, args);
			}
		}
		
		function close(lc: LocalConnection) {
			var doing = repr(lc) + ".close()";
			try {
				lc.close();
				trace(doing);
			} catch (e) {
				trace(doing + ": ! " + e);
			}
		}
		
		function addTest(name: String, fn: Function) {
			tests.push([name, fn]);
		}
		
		function createTestFunction(name: String) {
			return function() {
				trace(name + " was called with " + arguments.length + " argument" + (arguments.length == 0 ? "" : "s"));
				if (arguments.length > 0) {
					trace("  " + repr(arguments));
				}
			}
		}
		
		function setupEvents(lc: LocalConnection) {
			var name: String = repr(lc);
			lc.addEventListener(AsyncErrorEvent.ASYNC_ERROR, function(event: AsyncErrorEvent) {
				trace(name + " received event AsyncErrorEvent.ASYNC_ERROR");
				trace("  bubbles: " + repr(event.bubbles));
				trace("  cancelable: " + repr(event.cancelable));
				trace("  error: " + event.error);
				trace("  currentTarget: " + repr(event.currentTarget));
				trace("  target: " + repr(event.target));
				trace("");
			});
			lc.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(event: SecurityErrorEvent) {
				trace(name + " received event SecurityErrorEvent.SECURITY_ERROR");
				trace("  bubbles: " + repr(event.bubbles));
				trace("  cancelable: " + repr(event.cancelable));
				trace("  text: " + repr(event.text));
				trace("  currentTarget: " + repr(event.currentTarget));
				trace("  target: " + repr(event.target));
				trace("");
			});
			lc.addEventListener(StatusEvent.STATUS, function(event: StatusEvent) {
				trace(name + " received event StatusEvent.STATUS");
				trace("  bubbles: " + repr(event.bubbles));
				trace("  cancelable: " + repr(event.cancelable));
				trace("  code: " + repr(event.code));
				trace("  currentTarget: " + repr(event.currentTarget));
				trace("  level: " + repr(event.level));
				trace("  target: " + repr(event.target));
				trace("");
			});
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
			if (value === receiver) {
				return "receiver";
			} else if (value === sender) {
				return "sender";
			} else if (value === recvObject) {
				return "recvObject";
			} else if (value === custom) {
				return "custom";
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
