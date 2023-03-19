package {
    import flash.display.MovieClip;
    import flash.accessibility.Accessibility;
    public class Test extends MovieClip {
        public function Test() {
          trace(Accessibility.active);
        }
    }
}
