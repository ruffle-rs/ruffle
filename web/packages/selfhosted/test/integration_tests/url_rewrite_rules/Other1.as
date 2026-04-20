package {
    import flash.display.MovieClip;

    public class Other1 extends MovieClip {
        public function Other1() {
            trace("Loaded other1!");

            var value:String = loaderInfo.parameters["v"];
            trace("QP Value: " + value);
        }
    }
}
