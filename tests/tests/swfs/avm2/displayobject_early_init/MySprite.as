package  {
	
	import flash.display.Sprite;
	import flash.text.TextField;
	
	
	public class MySprite extends Sprite {
		
		
		public function MySprite() {
			var earlyGraphics = this.graphics;
			trace("MySprite before super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren + " this.getChildAt(0) = " + this.getChildAt(0) + " this.parent = " + this.parent);
			var textChild = new TextField();
			textChild.text = "Text before super()";
			this.addChild(textChild);
			super();
			trace("MySprite after super(): this.graphics === earlyGraphics: " + (this.graphics === earlyGraphics) + " this.numChildren = " + this.numChildren + " this.getChildAt(0) = " + this.getChildAt(0) + " this.parent = " + this.parent);
		}
	}
	
}
