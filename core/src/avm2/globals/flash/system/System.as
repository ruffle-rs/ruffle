package flash.system {
    [Ruffle(Abstract)]
    public final class System {
        import __ruffle__.stub_method;

        public static function gc():void {}

        public static function pauseForGCIfCollectionImminent(imminence:Number = 0.75):void {
            stub_method("flash.system.System", "pauseForGCIfCollectionImminent");
        }

        public static native function setClipboard(string:String):void;

        public static function disposeXML(node:XML):void {
            stub_method("flash.system.System", "disposeXML");
        }

        public static native function get freeMemory():Number;

        public static native function get privateMemory():Number;

        public static native function get totalMemoryNumber():Number;

        public static function get totalMemory():uint {
            return totalMemoryNumber as uint;
        }
    }
}
