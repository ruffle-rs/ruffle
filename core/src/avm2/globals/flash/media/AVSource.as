package flash.media {
    import flash.events.EventDispatcher;

    [API("688")]
    public class AVSource extends EventDispatcher {
        public function AVSource() {
            super();
        }

        public function dispose():AVResult {
            return new AVResult(AVResult.SUCCESS);
        }
    }
}
