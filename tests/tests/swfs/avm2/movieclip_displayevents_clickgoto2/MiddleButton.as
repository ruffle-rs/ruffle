package {
	import flash.events.MouseEvent;
	import flash.events.Event;
	
	public class MiddleButton extends EventWatcher {
		public function MiddleButton() {
			super();
			
			this.addEventListener(MouseEvent.CLICK, this.clicked);
			this.stop();
		}
	
		public function clicked(event: Event) {
			trace("///(MiddleButton clicked...)");
			trace("///this.parent.r_button.gotoAndPlay(1);");
			this.parent.r_button.gotoAndPlay(1);
			
			trace("///this.parent.l_button.gotoAndPlay(2);");
			this.parent.l_button.gotoAndPlay(2);
		}
	}
}