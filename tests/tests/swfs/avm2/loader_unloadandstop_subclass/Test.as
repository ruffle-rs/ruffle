package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            // https://github.com/ruffle-rs/ruffle/issues/25133:
            // `unloadAndStop` must not call the overridden `unload()`.
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
        // Stop before a stack overflow crashes the test suite.
        if (unloadCount > 5) {
            return;
        }
        super.unloadAndStop(false);
    }
}
