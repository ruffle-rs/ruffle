package flash.filters {
	public class BitmapFilter {
		public function clone(): BitmapFilter {
			throw new Error("BitmapFilter.clone() must be overridden!")
		}
	}
}