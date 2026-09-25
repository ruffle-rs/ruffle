package {
    import flash.display.MovieClip;

    public class Load2 extends MovieClip {
        public function Load2() {
            var self:Load2 = this;
            this.addEventListener("addedToStage", function() {
                trace("Loaded #2- url: " + Test.processURL(self.loaderInfo.url));
                trace("Loaded #2- loader url: " + Test.processURL(self.loaderInfo.loaderURL));
            });
        }
    }
}
