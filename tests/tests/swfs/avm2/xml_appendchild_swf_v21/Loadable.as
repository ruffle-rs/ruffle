// should be compiled as SWF Version 21

package {
    import flash.display.Sprite;

    public class Loadable extends Sprite {
        public function Loadable() {
            var x = new XML('<root></root>');
            var x1 = new XML('<a>Test</a>');
            x.appendChild(x1);
            trace("[loadable.swf] " + x);
            // If SWF version 21 or higher the node will be removed from the previous parent before its appended.
            x.appendChild(x.child(0)[0]);
            trace("[loadable.swf] " + x);
        }
    }
}