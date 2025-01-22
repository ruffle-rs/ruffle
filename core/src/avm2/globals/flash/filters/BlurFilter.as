package flash.filters {
    public final class BlurFilter extends BitmapFilter {
        [Ruffle(NativeAccessible)]
        public var blurX: Number;

        [Ruffle(NativeAccessible)]
        public var blurY: Number;

        [Ruffle(NativeAccessible)]
        public var quality: int;

        public function BlurFilter(blurX: Number = 4.0, blurY: Number = 4.0, quality: int = 1) {
            this.blurX = blurX;
            this.blurY = blurY;
            this.quality = quality;
        }

        override public function clone(): BitmapFilter {
            return new BlurFilter(this.blurX, this.blurY, this.quality);
        }
    }
}
