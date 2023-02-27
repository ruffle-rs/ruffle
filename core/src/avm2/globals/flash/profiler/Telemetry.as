package flash.profiler {
    public final class Telemetry {
        public static const connected: Boolean = false;
        public static const spanMarker: Number = 0;

        public static function sendMetric(metric:String, value:*):void {}
        public static function sendSpanMetric(metric:String, startSpanMarker:Number, value:* = null):void {}
        	
        public static function registerCommandHandler(commandName:String, handler:Function):Boolean {
            return false;
        }

        public static function unregisterCommandHandler(commandName:String):Boolean {
            return false;
        }
    }
}