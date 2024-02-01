package {
	import flash.display.Loader;
	import flash.events.Event;
	import flash.events.ProgressEvent;
	import flash.events.IOErrorEvent;
	import flash.net.URLRequest;
	import flash.display.MovieClip;

	public class Test {
		public function Test(main: MovieClip) {
			var loader = new Loader();
			loader.contentLoaderInfo.addEventListener(Event.OPEN, function(e) {
				printEvent(Event.OPEN, e, loader);
			});
			loader.contentLoaderInfo.addEventListener(Event.INIT, function(e) {
				printEvent(Event.INIT, e, loader);
			});
			loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, function(e) {
				printEvent(IOErrorEvent.IO_ERROR, e, loader);
			});
			loader.contentLoaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e) {
				printEvent(ProgressEvent.PROGRESS, e, loader);
			});
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				printEvent(Event.COMPLETE, e, loader);
			});
			main.addChild(loader);
			loader.load(new URLRequest("data.txt"));
		}
	
		private function printEvent(name: String, event: Event, loader: Loader) {
			var eventString = event.toString();
			// Replace the platform-specific path in the test output
			var index = eventString.indexOf("file:///");
			if (index != -1) {
				eventString = eventString.substr(0, index) + "file:///[[RUFFLE PATH]]";
			}
			trace("Event: " + name + " event: " + eventString);
			trace("Content: " + loader.content);
			trace("Bytes length: " + loader.contentLoaderInfo.bytes.length);
			trace("loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded);
			trace("loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal);
			try {
				trace("loader.contentLoaderInfo.frameRate = " + loader.contentLoaderInfo.frameRate);
			} catch (e) {
				trace("Caught error: " + e);
			}
		}
	}
}