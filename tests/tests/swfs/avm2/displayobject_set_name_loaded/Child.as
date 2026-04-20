package {
    import flash.display.MovieClip;

    public class Child extends MovieClip {
        public function Child() {
            var self:Child = this;
            super();

            try {
                this.name = "test";
            } catch(e:Error) {
                trace("Caught " + Object.prototype.toString.call(e) + "; code " + e.errorID);
            }

            addEventListener("addedToStage", function(e) {
                try {
                    self.name = "test";
                } catch(e:Error) {
                    trace("Caught " + Object.prototype.toString.call(e) + "; code " + e.errorID);
                }
            });
        }
    }
}
