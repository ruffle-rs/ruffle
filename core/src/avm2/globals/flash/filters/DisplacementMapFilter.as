package flash.filters {
	import flash.display.BitmapData;
	import flash.geom.Point;

	public final class DisplacementMapFilter extends BitmapFilter {
		public var alpha: Number;
		public var color: uint;
		public var componentX: uint;
		public var componentY: uint;
		public var mapBitmap: BitmapData;
		public var mapPoint: Point;
		public var mode: String;
		public var scaleX: Number;
		public var scaleY: Number;

		public function DisplacementMapFilter(mapBitmap:BitmapData = null,
											  mapPoint:Point = null,
											  componentX:uint = 0,
											  componentY:uint = 0,
											  scaleX:Number = 0.0,
											  scaleY:Number = 0.0,
											  mode:String = "wrap",
											  color:uint = 0,
											  alpha:Number = 0.0) {
			this.mapBitmap = mapBitmap;
			this.mapPoint = mapPoint;
			this.componentX = componentX;
			this.componentY = componentY;
			this.scaleX = scaleX;
			this.scaleY = scaleY;
			this.mode = mode;
			this.color = color;
			this.alpha = alpha;
		}

		override public function clone(): BitmapFilter {
			return new DisplacementMapFilter(this.mapBitmap.clone(), this.mapPoint.clone(), this.componentX, this.componentY, this.scaleX, this.scaleY, this.mode, this.color, this.alpha);
		}
	}
}
