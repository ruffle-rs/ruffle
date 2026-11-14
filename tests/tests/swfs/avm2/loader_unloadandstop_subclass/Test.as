package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            // Regression for https://github.com/ruffle-rs/ruffle/issues/25133:
            // A Loader subclass that overrides `unload()` and in that override calls
            // `super.unloadAndStop(false)` used to recurse indefinitely because Ruffle's
            // `unloadAndStop` invoked the virtual `this.unload()`, which dispatched back
            // to the subclass override.
            var loader:MyLoader = new MyLoader();
            loader.unload();
            trace("subclass unload count: " + loader.unloadCount);
        }
    }
}

import flash.display.Loader;

class MyLoader extends Loader {
    public var unloadCount:int = 0;

    public function MyLoader() {
        super();
    }

    override public function unload():void {
        unloadCount++;
        // Use a simple counter instead of allowing the stack overflow which crashes the
        // test suite.
        if (unloadCount > 5) {
            return;
        }
        super.unloadAndStop(false);
    }
}
