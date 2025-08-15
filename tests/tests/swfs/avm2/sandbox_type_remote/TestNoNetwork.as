package {
    import flash.display.Sprite;
    import flash.system.Security;

    // Compile with network disabled as localhost/test-no-network.swf
    public class TestNoNetwork extends Sprite {
        public function TestNoNetwork() {
            trace("[No network] Current sandbox type: " + Security.sandboxType);
        }
    }
}
