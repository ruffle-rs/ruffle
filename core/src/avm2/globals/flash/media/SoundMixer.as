package flash.media {
    import flash.utils.ByteArray;
    
    public final class SoundMixer {

        public static native function get soundTransform():SoundTransform;
        public static native function set soundTransform(value:SoundTransform):void;

        public static native function get bufferTime():int;
        public static native function set bufferTime(value:int):void;

        public static native function stopAll():void;
        public static native function areSoundsInaccessible():Boolean
        public static native function computeSpectrum(outputArray:ByteArray, FFTMode:Boolean = false, stretchFactor:int = 0):void;
    }
}