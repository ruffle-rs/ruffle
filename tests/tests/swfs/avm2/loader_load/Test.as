package  {
	
	import flash.display.MovieClip;
	import flash.net.URLRequest;
	import flash.display.Loader;
	import flash.utils.ByteArray;
	import flash.net.URLVariables;
	import flash.events.Event;
	
	
	public class Test extends MovieClip {
		var currentTest: Number = 0;
		var requests: Array = [];
		
		public function Test() {
			var datas: Array = [];
			datas.push("foo");
			datas.push({toString: function() { return "baz"; }});
			datas.push("foo=bar");
			var vars = new URLVariables();
			vars.aaa = "bbb";
			vars.cccc = true;
			datas.push(vars);
			var ba: ByteArray = new ByteArray();
			ba.writeUTFBytes("a=b");
			datas.push(ba);
			
			for each (var method in ["post", "GET"]) {
				for each (var data in datas) {
					var request: URLRequest = new URLRequest();
					request.url = "http://example.org?foo";
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
