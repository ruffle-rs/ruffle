package {
    import flash.display.Sprite;
    import flash.system.Security;

    // Compile with network enabled
    public class Test extends Sprite {
        public function Test() {
            trace("Current sandbox type: " + Security.sandboxType);
        }
    }
}
