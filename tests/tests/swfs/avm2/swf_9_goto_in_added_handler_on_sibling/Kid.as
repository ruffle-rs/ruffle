package {
    import flash.display.MovieClip;

    public class Kid extends MovieClip {
        public function Kid() {
            trace("Kid constructor (parent " + (parent ? parent.name : null) + ")");
        }
    }
}
