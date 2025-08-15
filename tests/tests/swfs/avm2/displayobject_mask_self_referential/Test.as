// compiled with mxmlc


package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test(){
            var mc = new MovieClip();
            this.mask = this;
            this.mask = mc;
        }
    }
}