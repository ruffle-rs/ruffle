package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLRequest;

    public class Load1 extends MovieClip {
        public function Load1() {
            var self:Load1 = this;
            this.addEventListener("addedToStage", function() {
                trace("Loaded #1- url: " + Test.processURL(self.loaderInfo.url));
                trace("Loaded #1- loader url: " + Test.processURL(self.loaderInfo.loaderURL));

                var loader:Loader = new Loader();
                self.addChild(loader);
                loader.load(new URLRequest("load2.swf"));
            });
        }
    }
}
