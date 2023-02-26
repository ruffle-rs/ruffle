package flash.display {
    public class Shape extends DisplayObject {
        public function Shape() {
            this.init();
        }
        private native function init();

        public native function get graphics():Graphics;

        internal var _graphics:Graphics;
    }
}