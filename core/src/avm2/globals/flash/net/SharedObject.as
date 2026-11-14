package flash.net {
    import flash.events.EventDispatcher;

    [Ruffle(Abstract)]
    public class SharedObject extends EventDispatcher {
        public static var defaultObjectEncoding:uint = ObjectEncoding.DEFAULT;

        public static native function getLocal(
            name:String,
            localPath:String = null,
            secure:Boolean = false
        ):SharedObject;

        public static function getRemote(
            name:String,
            remotePath:String = null,
            persistence:Object = false,
            secure:Boolean = false
        ):SharedObject {
            return getLocal(name, remotePath, secure);
        }

        public static function deleteAll(url:String):int {
            return 0;
        }

        public static function getDiskUsage(url:String):int {
            return 0;
        }

        public native function get size():uint;
        public native function get objectEncoding():uint;
        public native function set objectEncoding(value:uint):void;

        public native function flush(minDiskSpace:int = 0):String;
        public native function close():void;
        public native function clear():void;

        public function connect(myConnection:NetConnection, params:String = null):void {
            if (myConnection == null) {
                Error.throwError(TypeError, 2007, "myConnection");
            }
        }

        public function send(...arguments):void {
        }

        public function setDirty(propertyName:String):void {
        }

        public function setFps(updatesPerSecond:Number):Boolean {
            return false;
        }

        public function setProperty(propertyName:String, value:Object = null):void {
            this.data[propertyName] = value;
        }

        public native function get data():Object;
    }
}
