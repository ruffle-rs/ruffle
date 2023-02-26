package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	
	
	public class MyMovieClip extends MovieClip {
		
		
		public function MyMovieClip() {
			var earlyGraphics = this.graphics;
			trace("MyMovieClip before super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren + " this.getChildAt(0) = " + this.getChildAt(0) + " this.parent = " + this.parent);
			var textChild = new TextField();
			textChild.text = "Text before super()";
			this.addChild(textChild);
			super();
			trace("MyMovieClip after super(): this.graphics === earlyGraphics: " + (this.graphics === earlyGraphics) + " this.numChildren = " + this.numChildren + " this.getChildAt(0) = " + this.getChildAt(0) + " this.parent = " + this.parent);
		}
	}
	
}
