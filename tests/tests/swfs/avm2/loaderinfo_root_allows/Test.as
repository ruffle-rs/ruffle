package {
    import flash.display.MovieClip;
    import flash.utils.setTimeout;

    public class Test extends MovieClip {
        public function Test() {
            var self:Test = this;
            setTimeout(function(e:*):* {
                trace(self.loaderInfo.childAllowsParent);
                trace(self.loaderInfo.parentAllowsChild);
            }, 0);
        }
    }
}
