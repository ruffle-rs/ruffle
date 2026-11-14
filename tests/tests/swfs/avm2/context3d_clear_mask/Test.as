package {
    import flash.display.Sprite;
    import flash.display3D.Context3DClearMask;

    public class Test extends Sprite {
        public function Test() {
            trace("ALL: " + Context3DClearMask.ALL);
            trace("COLOR: " + Context3DClearMask.COLOR);
            trace("DEPTH: " + Context3DClearMask.DEPTH);
            trace("STENCIL: " + Context3DClearMask.STENCIL);
        }
    }
}
