// should be compiled as SWF Version 20 or below.

package {
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            var x = new XML('<root></root>');
            var x1 = new XML('<a>Test</a>');
            x.appendChild(x1);
            trace("[test.swf] " + x);
            // If SWF version 21 or higher the node will be removed from the previous parent before its appended.
            x.appendChild(x.child(0)[0]);
            trace("[test.swf] " + x);

            var loader:Loader = new Loader();
            loader.load(new URLRequest("loadable.swf"));
        }
    }
}
