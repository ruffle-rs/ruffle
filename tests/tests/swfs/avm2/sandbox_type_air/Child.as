package {
    import flash.display.Sprite;
    import flash.system.Security;

    public class Child extends Sprite {
        public function Child() {
            trace("Child sandbox type: " + Security.sandboxType);
        }
    }
}
