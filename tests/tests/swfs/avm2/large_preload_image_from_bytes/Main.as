package {
	import flash.display.Stage;
	import flash.display.Loader;
	import flash.display.Loader;
	import flash.net.URLRequest;
	import flash.errors.IllegalOperationError;
	import flash.display.Sprite;
	import flash.events.Event;
	import flash.events.ProgressEvent;
	import flash.events.HTTPStatusEvent;
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	import flash.display.Bitmap;
			
	public class Main extends MovieClip {

		[Embed(source="5000x5000.png", mimeType="application/octet-stream")]
		private static var LOADABLE_IMAGE_BYTES:Class;
		private var loader: Loader;
		
		public function Main() {		
			this.setupLoader();
			trace("Calling super() in Main()");
			super();
			trace("Called super() in Main()");
			
			var self = this;
			
			this.addEventListener(Event.ENTER_FRAME, function(e) {
				// FIXME - re-enable this when the timing of 'content' being
				// set in Ruffle matches Flash Player
				//trace("enterFrame in Test: this.loader.content = " + self.loader.content);
			});
		
			this.addEventListener(Event.EXIT_FRAME, function(e) {
				trace("exitFrame in Test");
			});
		}
	
		private function dumpParams(obj: Object) {
			var out = []
			for (var key in obj) {
				out.push(key + " = " + obj[key]);
			}
			out.sort();
			trace("Parameters: (len=" + out.length + ")");
			trace(out);
		}
	
		private function dumpLoader(loader: Loader) {
			trace("loader.content = " + loader.content);
			trace("loader.contentLoaderInfo.content = " + loader.contentLoaderInfo.content);
			trace("loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded);
			trace("loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal);
			//trace("loader.contentLoaderInfo.bytes?.length = " + (loader.contentLoaderInfo.bytes ? loader.contentLoaderInfo.bytes.length : null)); 
			trace("loader.contentLoaderInfo.url = " + loader.contentLoaderInfo.url);
			trace("loader.contentLoaderInfo.parameters = " + loader.contentLoaderInfo.parameters);		
		}
	
		function setupLoader() {
			this.loader = new Loader();
			this.addChild(loader);
			this.dumpLoader(loader);


			function dump(event:Event) {
				var url = loader.contentLoaderInfo.url;
				if (url) {
					// This truncates the path to 'file:///' to make the output
					// reproducible across deifferent machines 
					url = url.substr(0, 8);
				}
				trace("Event " + event + ": "
					+ "loader.numChildren = " + loader.numChildren
					+ ", loader.content = " + loader.content 
					+ ", loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded
					+ ", loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal
					// TODO - enable this when we correctly construct a fake SWF for the image
					//+ ", loader.contentLoaderInfo.bytes.length = " + loader.contentLoaderInfo.bytes.length
					+ ", loader.contentLoaderInfo.url = " + url);
			}

			loader.contentLoaderInfo.addEventListener(Event.OPEN, function(e) {
				dump(e);
			});
		
			loader.contentLoaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e) {
				// FIXME - the 'bytesLoaded' and 'bytesTotal' values printed here are wrong,
				// as they are not properly implemented in Ruffle. Once the implementation is fixed,
				// the output of this test will change.
				dump(e);
			});

			loader.contentLoaderInfo.addEventListener(Event.INIT, function(e) {
				dump(e);
				trace("Init: loader.content = " + loader.content);
				trace("Init: loader.content.bitmapData = " + Bitmap(loader.content).bitmapData);
			});

			loader.contentLoaderInfo.addEventListener(HTTPStatusEvent.HTTP_STATUS, function(e) {
				dump(e);
			});
			
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				dump(e);
			});

			loader.loadBytes(ByteArray(new LOADABLE_IMAGE_BYTES()));
			trace("Directly after load:");
			this.dumpLoader(loader);
		
			// Enable this to print and save the fake SWF
			/*var bytes = loader.contentLoaderInfo.bytes;
			new FileReference().save(bytes);
			var readBack = [];
			for (var i = 0; i < 64; i++) {
				readBack.push(bytes[i]);;
			}
			trace(readBack);*/
		
			return loader;
		}
	}
}