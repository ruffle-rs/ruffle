package  {
	
	import flash.display.SimpleButton;
	
	
	public class MyButton extends SimpleButton {
		
		
		public function MyButton() {
			// FIXME - test button states when we fix SimpleButton construction to match Flash Player
			trace("MyButton before super(): this.mouseEnabled = " + this.mouseEnabled + " this.parent = " + this.parent);
			super();
			trace("MyButton after super(): this.mouseEnabled = " + this.mouseEnabled + " this.parent = " + this.parent);
		}
	}
	
}
