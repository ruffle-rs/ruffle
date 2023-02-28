package  {
	import flash.net.*;
	import flash.display.Loader;
	import flash.system.LoaderContext;
	import flash.system.ApplicationDomain;
	import flash.utils.ByteArray;
	import flash.utils.getDefinitionByName;
	public class Test {

		public function Test(s) {
			var myURLReq: URLRequest = new URLRequest();
			myURLReq.url = "framework_4.5.0.20967.swz";
			myURLReq.digest = "9f67b1c289a5b5db7b32844af679e758541d101b46a7f75672258953804971ff";
			var myURLLoader: URLLoader = new URLLoader();
			myURLLoader.dataFormat = URLLoaderDataFormat.BINARY;
			myURLLoader.addEventListener("complete", onC);

			myURLLoader.load(myURLReq);

			function onC(e) {
				var someLoader:Loader = new Loader();
				var context: LoaderContext = new LoaderContext();
				context.applicationDomain = ApplicationDomain.currentDomain;
				s.addChild(someLoader);
				someLoader.loadBytes((ByteArray)(myURLLoader.data), context); 
				someLoader.contentLoaderInfo.addEventListener("init", init);
				
				
			}
		
			function init(e) {
				// verify that the classes loaded from the SWZ is now in our current domain.
				trace(getDefinitionByName("mx.events.PropertyChangeEvent"));
				trace(getDefinitionByName("mx.core.ByteArrayAsset"));
				trace(getDefinitionByName("Test"));
				
				// Try without context
				var anotherLoader:Loader = new Loader();
				s.addChild(anotherLoader);
				anotherLoader.loadBytes((ByteArray)(myURLLoader.data));
				anotherLoader.contentLoaderInfo.addEventListener("init", init2);
			}
		
			function init2(e) {
				// Confirm that the loaded movieclip still has access to it's parent domain despite 
				// never specifying the domain.
				trace(e.target.applicationDomain.getDefinition("Test"));
			}

		}

	}
	
}
