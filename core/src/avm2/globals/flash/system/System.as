package flash.system {
    [Ruffle(Abstract)]
    public final class System {

        public static function gc():void {}

        [API("674")]
        public static function pauseForGCIfCollectionImminent(imminence:Number = 0.75):void {
        }

        public static native function setClipboard(string:String):void;

        [API("662")]
        public static function disposeXML(node:XML):void {
        }

        public static native function exit(code:uint):void;

        public static function get freeMemory():Number {
            return 1024*1024*10; // 10MB
        }

        public static function get privateMemory():Number {
            return 1024*1024*100; // 100MB
        }

        public static function get totalMemoryNumber():Number {
            return 1024*1024*90; // 90MB
        }

        public static function get totalMemory():uint {
            return totalMemoryNumber as uint;
        }
        
        public static var useCodePage:Boolean = false;
        
        public static function get vmVersion():String { 
            return "2.1"; 
        }
    }
}
