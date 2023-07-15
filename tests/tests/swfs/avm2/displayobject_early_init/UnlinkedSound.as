package {
	import flash.media.Sound;

	public class UnlinkedSound extends Sound {
		public function UnlinkedSound() {
			trace("UnlinkedSound before super(): this.bytesLoaded = " + this.bytesLoaded);
			super();
			trace("UnlinkedSound after super(): this.bytesLoaded = " + this.bytesLoaded);
		}
	}
}