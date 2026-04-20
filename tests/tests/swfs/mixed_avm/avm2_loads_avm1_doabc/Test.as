package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            var l:Loader = new Loader();
            l.load(new URLRequest("avm1.swf"));
            addChild(l);

            trace("Finished constructor of class of root MovieClip");
        }
    }
}
