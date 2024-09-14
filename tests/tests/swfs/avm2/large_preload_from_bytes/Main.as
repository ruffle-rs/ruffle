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
			
	public class Main extends MovieClip {

		[Embed(source="large_bytearray/test.swf", mimeType="application/octet-stream")]
		private static var LOADABLE_SWF:Class;
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
			trace("loader.contentLoaderInfo.bytes?.length = " + (loader.contentLoaderInfo.bytes ? loader.contentLoaderInfo.bytes.length : null)); 
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
					+ ", loader.contentLoaderInfo.bytes.length = " + loader.contentLoaderInfo.bytes.length
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
				trace("loader.contentLoaderInfo === loader.content.loaderInfo : " + (loader.contentLoaderInfo === loader.content.loaderInfo).toString());
				trace("loader.contentLoaderInfo.content === loader.content : " + (loader.contentLoaderInfo.content == loader.content).toString());
				dump(e);
			});

			loader.contentLoaderInfo.addEventListener(HTTPStatusEvent.HTTP_STATUS, function(e) {
				dump(e);
			});
			
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				dump(e);
			});

			loader.loadBytes(ByteArray(new LOADABLE_SWF()));
			//loader.load(new URLRequest("./large_bytearray/test.swf"));
			trace("Directly after load:");
			this.dumpLoader(loader);
			return loader;
		}
	}
}