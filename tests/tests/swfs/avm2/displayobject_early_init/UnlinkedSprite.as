package {
	import flash.display.Sprite;

	public class UnlinkedSprite extends Sprite {
		public function UnlinkedSprite() {
			trace("UnlinkedSprite before super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren);
			super();
			trace("UnlinkedSprite after super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren);
		}
	}
}