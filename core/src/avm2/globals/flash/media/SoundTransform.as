package flash.media {
	public final class SoundTransform {
        public var leftToLeft:Number;
        public var leftToRight:Number;
        public var rightToLeft:Number;
        public var rightToRight:Number;
        public var volume:Number;

        public function SoundTransform(vol:Number = 1, panning:Number = 0) {
            this.volume = vol;
            this.pan = panning;
        }

        public native function get pan():Number;
        public native function set pan(value:Number):void;
    }
}