package flash.system {
    public final class System {
        import __ruffle__.stub_method;

        public static function gc(): void {

        }

        public static function pauseForGCIfCollectionImminent(imminence:Number = 0.75): void {
            stub_method("flash.system.System", "pauseForGCIfCollectionImminent");
        }

        public static native function setClipboard(string:String): void;

        public static function disposeXML(node:XML):void {
            stub_method("flash.system.System", "disposeXML");
        }
    }
}
