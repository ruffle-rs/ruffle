package  {
	
	import flash.display.MovieClip;
	import flash.display.SimpleButton;
	
	
	public class MyButton extends SimpleButton {
		
		
		public function MyButton() {
			trace("Calling MyButton super()");
			super();
			trace("Called MyButton super()");
		}
	}
	
}
