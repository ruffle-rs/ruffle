package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLLoader;
    import flash.net.URLRequest;
    import flash.system.ApplicationDomain;
    import flash.system.LoaderContext;

    public class Test extends MovieClip {
        public var originalURL:String;
        public var counter:int;

        public function Test() {
            // Recursively loads this SWF into itself using loadBytes
            var self:Test = this;

            trace("Loaded new SWF!");
            addEventListener("addedToStage", function(e) {
                var rootInstance:Object = self.stage.getChildAt(0);

                if (rootInstance.counter == 0) {
                    // This code is running on the root. Initialize the
                    // properties.
                    rootInstance.originalURL = self.loaderInfo.url;
                    rootInstance.counter = 1;
                }

                // Output the current SWF's info
                trace("    url: " + self.processURL(self.loaderInfo.url));
                trace("    loader url: " + self.processURL(self.loaderInfo.loaderURL));

                // Update the url and counter for this SWF in the chain
                rootInstance.counter ++;

                if (rootInstance.counter === 5) {
                    return;
                }

                var req:URLRequest = new URLRequest(rootInstance.originalURL);
                var urlLoader:URLLoader = new URLLoader();
                urlLoader.dataFormat = "binary";
                urlLoader.addEventListener("complete", function(e) {
                    var l:Loader = new Loader();
                    self.addChild(l);

                    // If we don't create a new ApplicationDomain for each load,
                    // the code attempts to register multiple Test classes in
                    // the same registry
                    l.loadBytes(urlLoader.data, new LoaderContext(false, new ApplicationDomain()));
                });
                urlLoader.load(req);
            });
        }

        public function processURL(url:String) {
            var rootInstance:Object = this.stage.getChildAt(0);

            var sliced:String = url.slice(rootInstance.originalURL.length);
            // Remove FP's non-deterministic counter output
            return sliced.replace(/\/\d/g, "/<id>");
        }
    }
}
