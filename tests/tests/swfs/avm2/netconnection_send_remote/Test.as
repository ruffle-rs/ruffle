package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	import flash.events.NetStatusEvent;
	import flash.net.NetConnection;
	import flash.net.ObjectEncoding;
	import flash.net.Responder;
	import flash.system.fscommand;
	

	public class Test extends MovieClip {
		var connection: NetConnection = new NetConnection();
		var currentTest: uint = 0;
		var expectedSuccesses: int = 0;
		var expectedStatusEvents: int = 0;

		public function Test() {
			addEventListener(Event.ENTER_FRAME, onFrame);
			connection.addEventListener(NetStatusEvent.NET_STATUS, onNetStatus);

			// At time of writing, ruffle doesn't support AMF3 properly
			connection.objectEncoding = ObjectEncoding.AMF0;
		}

		function onNetStatus(event: NetStatusEvent) {
			expectedStatusEvents--;

			var keys = [];
			for (var key in event.info) {
				keys.push(key);
			}
			keys.sort();
			for each (var key in keys) {
				trace("[onNetStatus] event.info." + key + " = " + event.info[key]);
			}
		}

		function onFrame(event: Event): void {
			if (expectedSuccesses == 0 && expectedStatusEvents == 0) {
				if (currentTest > 0) {
					trace("# End of Test " + currentTest);
				}
				currentTest++;
				if (("test" + currentTest) in this) {
					if (currentTest > 1) {
						trace("");
					}
					trace("# Start of Test " + currentTest);

					// We connect before every test to reset Flash's internal counter of response IDs.
					// It has no actual impact on the NetConnection, but it keeps the bytes deterministic enough to be
					// responded to.
					// It also helps us set the specific test response in Ruffle.
					connection.connect("http://localhost:8000/test" + currentTest);
					expectedSuccesses = 0;
					expectedStatusEvents = 0;
					this["test" + currentTest]();
				} else {
					fscommand("exit");
					removeEventListener(Event.ENTER_FRAME, onFrame);
				}
			}
		}

		public function test1() {
			expectedSuccesses = 1;
			connection.call("test.method", createResponder("onResult works", "onResult", "Success!"), "Argument 1", true, 123, {key: "Hello World!"});
		}

		public function test2() {
			expectedSuccesses = 1;
			connection.call("test.method", createResponder("onStatus works", "onStatus", -123));
		}

		public function test3() {
			expectedSuccesses = 2;
			connection.call("test.method", createResponder("Call 1 with headers", "onStatus", false));

			// To show that headers set in the middle of a batch of calls, affect the whole batch
			connection.addHeader("Required", true, "value");

			// To show that the documentation of "addHeader(name) deletes the header" is false
			connection.addHeader("Duplicate", true, "original");
			connection.addHeader("Duplicate");

			connection.call("test.method", createResponder("Call 2 with headers", "onResult", true));
		}

		public function test4() {
			expectedStatusEvents = 1;

			// This one will give a HTTP 404, see what happens!
			connection.call("failure", createResponder("Expected failure", "", null));
		}

		function createResponder(name: String, expectedMethod: String, expectedValue: *): Responder {
			trace("[" + name + "] started");

			var test = function(actualMethod: String, actualValue: *) {
				if (expectedMethod == actualMethod) {
					if (expectedValue == actualValue) {
						trace("[" + name + "] passed!");
						expectedSuccesses--;
					} else {
						trace("[" + name + "] failed!");
						trace("Expected: " + expectedValue);
						trace("Actual: " + actualValue);
					}
				} else {
					trace("[" + name + "] failed! " + actualMethod + " called, " + expectedMethod + " expected");
				}
			};

			return new Responder(function(actualValue) {
				test("onResult", actualValue);
			}, function(actualValue) {
				test("onStatus", actualValue);
			});
		}
	}
}
