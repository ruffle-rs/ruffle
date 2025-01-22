package flash.display {

    [Ruffle(InstanceAllocator)]
    public class Shape extends DisplayObject {

        [Ruffle(NativeAccessible)]
        private var _graphics:Graphics;

        public native function get graphics():Graphics;
    }
}
