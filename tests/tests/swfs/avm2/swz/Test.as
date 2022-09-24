package {
	import flash.net.*;
	import flash.display.Loader;
	import flash.utils.ByteArray;
	import flash.system.ApplicationDomain;
	public class Test{
		public function Test(s) {

			var myURLReq: URLRequest = new URLRequest();
			myURLReq.url = "framework_4.5.0.20967.swz";
			myURLReq.digest = "9f67b1c289a5b5db7b32844af679e758541d101b46a7f75672258953804971ff";
			var myURLLoader: URLLoader = new URLLoader();
			myURLLoader.dataFormat = URLLoaderDataFormat.BINARY;
			myURLLoader.addEventListener("complete", onC);

			myURLLoader.load(myURLReq);

			function onC(e) {
				var someLoader: Loader = new Loader();
				s.addChild(someLoader);
				someLoader.loadBytes((ByteArray)(myURLLoader.data));
				someLoader.contentLoaderInfo.addEventListener("init", init);
			}

			function init(e) {
				var domain: ApplicationDomain = e.target.applicationDomain;
				trace(domain.getDefinition("mx.core.ByteArrayAsset"));
				trace(domain.getDefinition("mx.core.BitmapAsset"));
			}
		}
	}
}