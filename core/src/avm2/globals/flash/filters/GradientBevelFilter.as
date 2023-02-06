package flash.filters {
	public final class GradientBevelFilter extends BitmapFilter {
		public var alphas : Array;
		public var angle : Number;
		public var blurX : Number;
		public var blurY : Number;
		public var colors : Array;
		public var distance : Number;
		public var knockout : Boolean;
		public var quality : int;
		public var ratios : Array;
		public var strength : Number;
		public var type : String;

		public function GradientBevelFilter(
			distance:Number = 4.0,
			angle:Number = 45,
			colors:Array = null,
			alphas:Array = null,
			ratios:Array = null,
			blurX:Number = 4.0,
			blurY:Number = 4.0,
			strength:Number = 1,
			quality:int = 1,
			type:String = "inner",
			knockout:Boolean = false
		) {
			this.distance = distance;
			this.angle = angle;
			this.colors = colors;
			this.alphas = alphas;
			this.ratios = ratios;
			this.blurX = blurX;
			this.blurY = blurY;
			this.strength = strength;
			this.quality = quality;
			this.type = type;
			this.knockout = knockout;
		}

		override public function clone(): BitmapFilter {
			return new GradientBevelFilter(
				this.distance,
				this.angle,
				this.colors,
				this.alphas,
				this.ratios,
				this.blurX,
				this.blurY,
				this.strength,
				this.quality,
				this.type,
				this.knockout
			);
		}
	}
}
