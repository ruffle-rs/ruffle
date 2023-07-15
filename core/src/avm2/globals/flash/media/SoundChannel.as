package flash.media {
    import flash.events.EventDispatcher;
    
    [Ruffle(InstanceAllocator)]
    public final class SoundChannel extends EventDispatcher {
        public native function get leftPeak():Number;
        public native function get rightPeak():Number;
        public native function get position():Number;
        public native function get soundTransform():SoundTransform;
        public native function set soundTransform(value:SoundTransform):void;
        public native function stop():void;
    }
}