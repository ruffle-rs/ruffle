package flash.media {
    [Ruffle(InstanceAllocator)]
    public final class SoundTransform {
        public function SoundTransform(vol:Number = 1, panning:Number = 0) {
            this.volume = vol;
            this.pan = panning;
        }

        [Ruffle(FastCall)]
        public native function get leftToLeft():Number;
        [Ruffle(FastCall)]
        public native function set leftToLeft(value:Number):void;

        [Ruffle(FastCall)]
        public native function get leftToRight():Number;
        [Ruffle(FastCall)]
        public native function set leftToRight(value:Number):void;

        [Ruffle(FastCall)]
        public native function get rightToLeft():Number;
        [Ruffle(FastCall)]
        public native function set rightToLeft(value:Number):void;

        [Ruffle(FastCall)]
        public native function get rightToRight():Number;
        [Ruffle(FastCall)]
        public native function set rightToRight(value:Number):void;

        [Ruffle(FastCall)]
        public native function get volume():Number;
        [Ruffle(FastCall)]
        public native function set volume(volume:Number):void;

        [Ruffle(FastCall)]
        public native function get pan():Number;
        [Ruffle(FastCall)]
        public native function set pan(value:Number):void;
    }
}
