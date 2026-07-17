// NOTE: Be sure to compile this SWF with the `localWithNetwork` sandbox type!
package {
    import flash.display.MovieClip;
    import flash.system.Security;

    public class Loaded extends MovieClip {
        public function Loaded() {
            trace("This SWF is of sandbox type " + Security.sandboxType);
        }
    }
}
