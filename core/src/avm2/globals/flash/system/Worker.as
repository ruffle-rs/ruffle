package flash.system {
    import flash.events.EventDispatcher;
    import flash.system.MessageChannel;

    [API("682")]
    [Ruffle(Abstract)]
    public final class Worker extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        public static native function get current():Worker;

        public native function get state():String;
        public native function get isPrimordial():Boolean;
        public native function createMessageChannel(receiver:Worker):MessageChannel;
        public native function setSharedProperty(key:String, value:*):void;
        public native function getSharedProperty(key:String):*;
        public native function start():void;
        public native function terminate():Boolean;
    }
}
