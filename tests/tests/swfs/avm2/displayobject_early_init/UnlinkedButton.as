package  {
	
	import flash.display.SimpleButton;
	
	
	public class UnlinkedButton extends SimpleButton {
		
		
		public function UnlinkedButton() {
			// FIXME - test button states when we fix SimpleButton construction to match Flash Player
			trace("UnlinkedButton before super(): this.mouseEnabled = " + this.mouseEnabled);
			super();
			trace("UnlinkedButton after super(): this.mouseEnabled = " + this.mouseEnabled);
		}
	}
	
}
