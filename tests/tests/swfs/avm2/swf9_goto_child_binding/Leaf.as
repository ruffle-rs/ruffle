package {
    import flash.display.MovieClip;

    public class Leaf extends MovieClip {
        public function Leaf() {
            Test.constructions++;
            trace("leaf constructor #" + Test.constructions);
        }
    }
}
