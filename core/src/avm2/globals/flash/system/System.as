package flash.system {
    [Ruffle(Abstract)]
    public final class System {
        import __ruffle__.stub_method;
        import __ruffle__.stub_getter;

        public static function gc(): void {

        }

        public static function pauseForGCIfCollectionImminent(imminence:Number = 0.75): void {
            stub_method("flash.system.System", "pauseForGCIfCollectionImminent");
        }

        public static native function setClipboard(string:String): void;

        public static function disposeXML(node:XML):void {
            stub_method("flash.system.System", "disposeXML");
        }

        public static function get freeMemory(): Number {
            stub_getter("flash.system.System", "freeMemory");
            return 1024*1024*10; // 10MB
        }

        public static function get privateMemory(): Number {
            stub_getter("flash.system.System", "privateMemory");
            return 1024*1024*100; // 100MB
        }

        public static function get totalMemoryNumber(): Number {
            stub_getter("flash.system.System", "totalMemoryNumber");
            return 1024*1024*90; // 90MB
        }

        public static function get totalMemory(): uint {
            return totalMemoryNumber as uint;
        }
    }
}
