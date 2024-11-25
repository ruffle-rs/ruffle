package flash.display {

    [Ruffle(InstanceAllocator)]
    public class Shape extends DisplayObject {

        [Ruffle(InternalSlot)]
        private var _graphics:Graphics;

        public native function get graphics():Graphics;
    }
}
