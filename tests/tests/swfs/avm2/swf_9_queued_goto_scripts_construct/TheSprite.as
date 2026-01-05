package {
    import flash.display.MovieClip;

    public class TheSprite extends MovieClip {
        public function TheSprite() {
            trace("Start of TheSprite ctor");
            super();
            theSprite = this;
            trace("End of TheSprite ctor");
        }
    }
}
