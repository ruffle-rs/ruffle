package flash.filters {
	public final class ColorMatrixFilter extends BitmapFilter {
		private var _matrix: Array;

		public function ColorMatrixFilter(matrix: Array = null) {
			if (matrix == null) {
				matrix = [
					1, 0, 0, 0, 0,
					0, 1, 0, 0, 0,
					0, 0, 1, 0, 0,
					0, 0, 0, 1, 0
				];
			}
			this.matrix = matrix;
		}

		// From the Flash docs, we need to make a copy of the `Array`,
		// as modifying the `filter.matrix` directly should have no effect.
		// We call the method in the AS3 namespace, as some SWFS define a
		// extend `Array` and declare a *public* 'concat' method with a
		// different signature.

		public function get matrix(): Array {
			return this._matrix.AS3::concat();
		}

		public function set matrix(matrix:Array):void {
			this._matrix = matrix.AS3::concat();
		}

		override public function clone(): BitmapFilter {
			return new ColorMatrixFilter(this.matrix.AS3::concat());
		}
	}
}
