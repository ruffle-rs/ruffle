package flash.filters {
	public final class GlowFilter extends BitmapFilter {
		public var alpha: Number;
		public var blurX: Number;
		public var blurY: Number;
		public var color: uint;
		public var inner: Boolean;
		public var knockout: Boolean;
		public var quality: int;
		public var strength: Number;

		public function GlowFilter(color: uint = 0xFF0000, alpha: Number = 1.0, blurX: Number = 6.0, blurY: Number = 6.0, strength: Number = 2, quality: int = 1, inner: Boolean = false, knockout: Boolean = false) {
			this.alpha = alpha;
			this.blurX = blurX;
			this.blurY = blurY;
			this.color = color;
			this.inner = inner;
			this.knockout = knockout;
			this.quality = quality;
			this.strength = strength;
		}

		override public function clone(): BitmapFilter {
			return new GlowFilter(this.color, this.alpha, this.blurX, this.blurY, this.strength, this.quality, this.inner, this.knockout);
		}
	}
}