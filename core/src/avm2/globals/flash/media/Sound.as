package flash.media {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.net.URLRequest;
    
    [Ruffle(InstanceAllocator)]
    public class Sound extends EventDispatcher {
        public function Sound(stream:URLRequest = null, context:SoundLoaderContext = null) {
            this.init(stream, context)
        }
        private native function init(stream:URLRequest, context:SoundLoaderContext);


        public native function get bytesLoaded():uint;
        public native function get bytesTotal():int;
        public native function get isBuffering():Boolean;
        public native function get isURLInaccessible():Boolean;
        public native function get url():String;
        public native function get length():Number;
        public native function get id3():ID3Info;
        public native function play(startTime:Number = 0, loops:int = 0, sndTransform:SoundTransform = null):SoundChannel;
        public native function extract(target:ByteArray, length:Number, startPosition:Number = -1):Number;
        public native function close():void;
        public native function load(stream:URLRequest, context:SoundLoaderContext = null):void;
        [API("674")]
        public native function loadCompressedDataFromByteArray(bytes:ByteArray, bytesLength:uint):void;
        [API("674")]
        public native function loadPCMFromByteArray(bytes:ByteArray, samples:uint, format:String = "float", stereo:Boolean = true, sampleRate:Number = 44100.0):void
    }
}