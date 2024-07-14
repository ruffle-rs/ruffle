package flash.desktop {
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    [API("668")]
    public class NativeProcess extends EventDispatcher {
        public function NativeProcess() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function start(info:NativeProcessStartupInfo):void {
            stub_method("flash.desktop.NativeProcess", "start");
        }
    }
}
