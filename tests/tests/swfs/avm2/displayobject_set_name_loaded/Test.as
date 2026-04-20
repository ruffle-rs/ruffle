package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            super();

            try {
                this.name = "test";
            } catch(e:Error) {
                trace("Caught " + Object.prototype.toString.call(e) + "; code " + e.errorID);
            }

            var l:Loader = new Loader();
            l.load(new URLRequest("child.swf"));
            addChild(l);
        }
    }
}
