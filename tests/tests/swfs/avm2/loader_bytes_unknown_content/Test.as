package {
	import flash.display.Loader;
	import flash.events.Event;
	import flash.events.ProgressEvent;
	import flash.events.IOErrorEvent;
	import flash.net.URLRequest;
	import flash.display.MovieClip;

	public class Test {
		[Embed(source = "data.txt", mimeType="application/octet-stream")]
		public static var DATA: Class;
		
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
			trace("Calling loadBytes");
			loader.loadBytes(new DATA());
			trace("Immediately after loadBytes:");
			trace("loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded);
			trace("loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal);
			trace("loader.contentLoaderInfo.bytes.length = " + loader.contentLoaderInfo.bytes.length);
		}
	
		private function printEvent(name: String, event: Event, loader: Loader) {
			var eventString = event.toString();
			trace("Event: " + name + " event: " + eventString);
			// FIXME - print 'bytesLoaded' and 'bytesTotal' when Ruffle properly matches Flash Player
			trace("Content: " + loader.content);
			trace("Bytes length: " + loader.contentLoaderInfo.bytes.length);
		}
	}
}