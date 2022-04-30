package {
	import flash.events.MouseEvent;
	import flash.events.Event;
	
	public class RightButton extends EventWatcher {
		public function RightButton() {
			super();
			
			this.addEventListener(MouseEvent.CLICK, this.clicked);
			this.stop();
		}
	
		public function clicked(event: Event) {
			trace("///(RightButton clicked...)");
			trace("///this.gotoAndPlay(1);");
			this.gotoAndPlay(1);
			
			trace("///this.parent.l_button.gotoAndPlay(2);");
			this.parent.l_button.gotoAndPlay(2);
		}
	}
}