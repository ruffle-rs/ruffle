package {
    import flash.display.Sprite;
    import flash.system.ApplicationDomain;
    public class Late extends Sprite {
        public function Late() {
            trace("late movie Collision=" + ApplicationDomain.currentDomain.getDefinition("shared.Collision").NAME);
        }
    }
}
