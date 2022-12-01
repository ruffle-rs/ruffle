package {
	import flash.display.Sprite;
	import flash.filters.BlurFilter;
	public class Test extends Sprite {
		public function Test() {
			trace("///this.filters.length == 0");
			trace(this.filters.length == 0)
			trace("///this.filters === this.filters");
			trace(this.filters === this.filters);
			trace("///this.filters = [new BlurFilter()]")
			this.filters = [new BlurFilter()];
			trace("///this.filters.length == 1");
			trace(this.filters.length == 1)
			trace("///this.filters = undefined")
			this.filters = undefined;
			trace("///this.filters.length == 0")
			trace(this.filters.length == 0)
			trace("///this.filters = null")
			this.filters = null;
			trace("///this.filters.length == 0")
			trace(this.filters.length == 0)
			try {
				trace("///this.filters = [1, 2, 3]")
				this.filters = [1, 2, 3];
			} catch (e: Error) {
				trace("Caught error: " + e);
			}
			try {
				trace("///this.filters = [new BlurFilter(), undefined]")
				this.filters = [new BlurFilter(), undefined];
			} catch (e: Error) {
				trace("Caught error: " + e);
			}
		}
	}
}
