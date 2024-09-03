package {
    import flash.display.Sprite;
    import flash.system.Security;

    // Compile with network enabled as localhost/test-network.swf
    public class TestNetwork extends Sprite {
        public function TestNetwork() {
            trace("[Network] Current sandbox type: " + Security.sandboxType);
        }
    }
}
