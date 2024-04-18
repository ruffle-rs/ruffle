package {
    import flash.net.URLRequest;
	import flash.net.URLVariables;
	import flash.net.navigateToURL;
	import flash.display.MovieClip;
	public class Test extends MovieClip {
        public function Test()
        {
            var request:URLRequest = new URLRequest("https://example.com/purchase/");
            var variables:URLVariables = new URLVariables();
            variables.sku = "Test";
            var empty_variables:URLVariables = new URLVariables();
            var cases:Array = [
                ["POST", null],
                ["GET", null],
                ["POST", empty_variables],
                ["GET", empty_variables],
                ["POST", variables],
                ["GET", variables],
                ["POST", "sku=Test"],
                ["GET", "sku=Test"]
            ];
            for each (var case_tuple:Array in cases) {
                var method:String = case_tuple[0];
                var data:Object = case_tuple[1];
                request.method = method;
                request.data = data;
                trace("// Method:", method);
                trace("// Data:", data);
                navigateToURL(request, "_blank");
                trace("");
            }
        }
	}
}
