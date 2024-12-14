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
			
	public class Test {
		var orphanLoader:Loader;
		
		public function Test(stage: Stage) {

			var test = this;

			var runOrphanLoader = function() {
				trace("Starting orphan Loader");
				this.orphanLoader = setupLoader(function() {});
			};
		
			var childLoader = this.setupLoader(runOrphanLoader);
			stage.addChild(childLoader);
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
			trace("loader.contentLoaderInfo.bytes = " + loader.contentLoaderInfo.bytes); 
			trace("loader.contentLoaderInfo.url = " + loader.contentLoaderInfo.url);
			trace("loader.contentLoaderInfo.parameters = " + loader.contentLoaderInfo.parameters);
			trace("loader.contentLoaderInfo.contentType = " + loader.contentLoaderInfo.contentType);
			dumpParams(loader.contentLoaderInfo.parameters);			
		}
	
		function setupLoader(done: Function) {
			var loader = new Loader();
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
					+ ", loader.contentLoaderInfo.url = " + url
					+ ", loader.contentLoaderInfo.contentType = " + loader.contentLoaderInfo.contentType);
				dumpParams(loader.contentLoaderInfo.parameters);
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
				done();
			});

			loader.load(new URLRequest("./loadable.swf?paramOne=ValOne&paramTwo=ValTwo"));
			trace("Directly after load:");
			this.dumpLoader(loader);
			return loader;
		}
	}
}