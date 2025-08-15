package  {
	
	import flash.display.MovieClip;
	import flash.net.URLRequest;
	import flash.display.Loader;
	import flash.utils.ByteArray;
	import flash.net.URLVariables;
	import flash.events.Event;
	import flash.events.IOErrorEvent;
	import flash.events.ProgressEvent;
	import flash.net.URLRequestHeader;
	import flash.events.SecurityErrorEvent;
	
	
	public class Test extends MovieClip {
		var currentTest: Number = 0;
		var requests: Array = [];
		
		public function Test() {
			var datas: Array = [];
			datas.push([]);
			datas.push({});
			datas.push("");
			var vars = new URLVariables();
			datas.push(vars);
			var ba: ByteArray = new ByteArray();
			datas.push(ba);
			
			for each (var method in ["POST"]) {
				for each (var data in datas) {
					var request: URLRequest = new URLRequest();
					request.url = "http://localhost:8000";
					request.method = method;
					request.data = data;
					requests.push(request);
				}
			}
			
			var req = {method: "DELETE"};
			req.__prototype__ = flash.net.URLRequest.prototype;
			requests.push(req);
			
			this.addEventListener(Event.ENTER_FRAME, this.onFrame);
		}
		
		function onFrame(event: Event) {
			if (currentTest == requests.length) return;
			if (currentTest > 0) trace("");
			trace("Test " + currentTest);
			var request = requests[currentTest++];
			load(request);
		}
		
		function load(request: URLRequest) {
			var loader = new Loader();
			loader.contentLoaderInfo.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(e) {
				trace("Security error: " + e);
			});
			/*loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, function(e) {
				trace("IO error: " + e);
			});*/
		
			trace("// request.url");
			trace(request.url);
			trace("");
			trace("// request.data");
			trace(request.data);
			trace("");
			trace("// request.method");
			trace(request.method);
			trace("");
			trace("// loader.load(request)");
			trace(loader.load(request));
		}
	}
	
}
