package {
    import flash.display.MovieClip;

    public class Comp extends MovieClip {
        public var text:String = "enter text";

        public function Comp() {
            trace("Comp constructor " + name);
        }

        override public function toString():String {
            return "[Comp " + text + "]";
        }
    }
}
