package flash.filters {
	public final class ColorMatrixFilter extends BitmapFilter {
		private var _matrix: Array;

		public function ColorMatrixFilter(matrix: Array = null) {
			this.matrix = matrix;
		}

		// From the Flash docs, we need to make a copy of the `Array`,
		// as modifying the `filter.matrix` directly should have no effect.

		public function get matrix(): Array {
			return this._matrix.concat();
		}

		public function set matrix(matrix:Array):void {
			this._matrix = matrix.concat();
		}

		override public function clone(): BitmapFilter {
			return new ColorMatrixFilter(this.matrix.concat());
		}
	}
}
