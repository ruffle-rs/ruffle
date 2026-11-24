package {
    import flash.display.MovieClip;

    public class Inner extends MovieClip {
        public function Inner() {
            trace("Inner constructor (parent " + (parent ? parent.name : null) + ")");
        }

        override public function toString():String {
            return "[Inner]";
        }
    }
}
