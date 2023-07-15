package flash.display {

    [Ruffle(InstanceAllocator)]
    public class Shape extends DisplayObject {
        public native function get graphics():Graphics;

        internal var _graphics:Graphics;
    }
}