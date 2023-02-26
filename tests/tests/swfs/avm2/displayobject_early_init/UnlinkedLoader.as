package {
	import flash.display.Loader;

	public class UnlinkedLoader extends Loader {
		public function UnlinkedLoader() {
			trace("UnlinkedLoader before super(): this.contentLoaderInfo = " + this.contentLoaderInfo + " this.numChildren = " + this.numChildren);
			super();
			trace("UnlinkedLoader after super(): this.contentLoaderInfo = " + this.contentLoaderInfo + " this.numChildren = " + this.numChildren);
		}
	}
}