package {
	import flash.net.URLStream;
	import flash.events.Event;
	import flash.events.IOErrorEvent;
	import flash.events.ProgressEvent;
	import flash.net.URLRequest;

	public class Test {
		public function Test() {
			var stream = new URLStream();
			stream.addEventListener(Event.OPEN, function(e) {
				printEvent(Event.OPEN, e, stream);
			});
			stream.addEventListener(IOErrorEvent.IO_ERROR, function(e) {
				printEvent(IOErrorEvent.IO_ERROR, e, stream);
			});
			stream.addEventListener(ProgressEvent.PROGRESS, function(e) {
				printEvent(ProgressEvent.PROGRESS, e, stream);
			});
			stream.addEventListener(Event.COMPLETE, function(e) {
				printEvent(Event.COMPLETE, e, stream);
				trace("Read string: " + stream.readUTFBytes(stream.bytesAvailable));
				trace("Bytes available: " + stream.bytesAvailable);
			});
			stream.load(new URLRequest("data.txt"));
		}

		private function printEvent(name: String, event: Event, stream: URLStream) {
			var eventString = event.toString();
			// Replace the platform-specific path in the test output
			var index = eventString.indexOf("file:///");
			if (index != -1) {
				eventString = eventString.substr(0, index) + "file:///[[RUFFLE PATH]]";
			}
			trace("Event: " + name + " event: " + eventString + " bytesAvailable: " + stream.bytesAvailable);

		}
	}
}