package flash.filters {
	public final class BlurFilter extends BitmapFilter {
		public var blurX: Number;
		public var blurY: Number;
		public var quality: int;

		public function BlurFilter(blurX: Number = 4.0, blurY: Number = 4.0, quality: int = 1) {
			this.blurX = blurX;
			this.blurY = blurY;
			this.quality = qualty;
		}

		override public function clone(): BitmapFilter {
			return new BlurFilter(this.blurX, this.blurY, this.quality);
		}
	}
}
