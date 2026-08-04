package flash.media {
    import __ruffle__.stub_constructor;
    import flash.events.EventDispatcher;

    [API("688")]
    public class AVStream extends EventDispatcher {
        public static const HARDWARE:String = "hardware";
        // Note: This is not a typo.
        public static const SOFTWARE:String = "sofware";
        public static const UNDEFINED:String = "undefined";

        public function AVStream(source:AVSource) {
            stub_constructor("flash.media.AVStream");
        }
    }
}
