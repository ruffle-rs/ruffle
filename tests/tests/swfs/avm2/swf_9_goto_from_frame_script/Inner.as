package {
    import flash.display.MovieClip;

    public class Inner extends MovieClip {
        public function Inner() {
            trace("Inner constructor (parent " + (parent ? parent.name : null) + ")");
            addFrameScript(0, frame1);
        }

        public function frame1():void {
            trace("Inner frame1 (parent " + parent.name + ")");
        }
    }
}
