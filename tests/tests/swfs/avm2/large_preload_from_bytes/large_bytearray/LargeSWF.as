package  {
	
	import flash.display.MovieClip;
	import flash.display.Loader;
	import flash.events.*;
	import flash.utils.ByteArray;
	
	
	public class LargeSWF extends MovieClip {
		
		private var loader: Loader;
		
		[Embed(source = "data1.bin", mimeType="application/octet-stream")]
		public static var DATA1: Class;
		
		[Embed(source = "data2.bin", mimeType="application/octet-stream")]
		public static var DATA2: Class;
		
		[Embed(source = "data3.bin", mimeType="application/octet-stream")]
		public static var DATA3: Class;
		
		[Embed(source = "data4.bin", mimeType="application/octet-stream")]
		public static var DATA4: Class;
		
		[Embed(source = "data5.bin", mimeType="application/octet-stream")]
		public static var DATA5: Class;
		
		[Embed(source= "../nested_load/test.swf", mimeType="application/octet-stream")]
		public static var NESTED_LOAD: Class;
		
		public function LargeSWF() {
			trace("Calling super() in LargeSWF()");
			super();
			trace("Called super() in LargeSWF()");
			trace("Loading ../nested_load/test.swf from bytes");
			this.setupLoader();
		}

	
		private function dumpLoader(loader: Loader) {
			trace("LargeSWF loader.content = " + loader.content);
			trace("LargeSWF loader.contentLoaderInfo.content = " + loader.contentLoaderInfo.content);
			trace("LargeSWF loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded);
			trace("LargeSWF loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal);
			trace("LargeSWF loader.contentLoaderInfo.bytes?.length = " + (loader.contentLoaderInfo.bytes ? loader.contentLoaderInfo.bytes.length : null)); 
			trace("LargeSWF loader.contentLoaderInfo.url = " + loader.contentLoaderInfo.url);
			trace("LargeSWF loader.contentLoaderInfo.parameters = " + loader.contentLoaderInfo.parameters);		
		}
	
		private function setupLoader() {
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
				trace("LargeSWF Event " + event + ": "
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
				trace("LargeSWF loader.contentLoaderInfo === loader.content.loaderInfo : " + (loader.contentLoaderInfo === loader.content.loaderInfo).toString());
				trace("LargeSWF loader.contentLoaderInfo.content === loader.content : " + (loader.contentLoaderInfo.content == loader.content).toString());
				dump(e);
			});

			loader.contentLoaderInfo.addEventListener(HTTPStatusEvent.HTTP_STATUS, function(e) {
				dump(e);
			});
			
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				dump(e);
			});

			loader.loadBytes(ByteArray(new NESTED_LOAD()));
			trace("LargeSWF: Directly after load:");
			this.dumpLoader(loader);
			return loader;
		}	
	}
	
}
