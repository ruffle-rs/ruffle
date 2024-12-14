package {
	import flash.display.MovieClip;
	import flash.utils.ByteArray;

	public class Test extends MovieClip {
		[Bindable]
		[Embed(source="loadable.swf", mimeType="application/octet-stream")]
		private var loadableSwf:Class;
		
		public function Test() {
			super();
			addFrameScript(0,this.frame1);
		}
	
		function urlPrefix(url: String): String {
			return url ? url.substring(0, 8) : url;
		}
		
		function frame1() {
			import flash.display.Loader;
			import flash.net.URLRequest;
			import flash.errors.IllegalOperationError;
			import flash.display.Sprite;
			import flash.events.Event;
			import flash.events.ProgressEvent;
			
			var self = this;
			var loader = new Loader();
			this.stage.addChild(loader);
			trace("loader.content = " + loader.content);
			trace("loader.contentLoaderInfo.content = " + loader.contentLoaderInfo.content);
			trace("loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded);
			trace("loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal);
			trace("loader.contentLoaderInfo.bytes = " + loader.contentLoaderInfo.bytes); 
			trace("loader.contentLoaderInfo.url = " + this.urlPrefix(loader.contentLoaderInfo.url));

			var bytes = ByteArray(new loadableSwf);
			
			function dump(event:Event) {
				trace("Event " + event + ": "
					+ "loader.numChildren = " + loader.numChildren
					+ ", loader.content = " + loader.content 
					+ ", loader.contentLoaderInfo.bytesLoaded = " + loader.contentLoaderInfo.bytesLoaded
					+ ", loader.contentLoaderInfo.bytesTotal = " + loader.contentLoaderInfo.bytesTotal
					+ ", loader.contentLoaderInfo.url = " + self.urlPrefix(loader.contentLoaderInfo.url));	
				trace("bytes.position = " + bytes.position);
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
			
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				dump(e);
				trace("Stage children before addChild attempt: " + self.stage.numChildren);
				trace("loader.numChildren before addChild attempt: " + loader.numChildren);
				trace("loader.content before addChild attempt: " + loader.content);
				self.stage.addChild(loader.content);
				trace("Stage children after addChild attempt: " + self.stage.numChildren);
				trace("loader.numChildren after addChild attempt: " + loader.numChildren);
				trace("loader.content after addChild attempt: " + loader.content);
			});
			
			loader.loadBytes(bytes);
		}
	}
}