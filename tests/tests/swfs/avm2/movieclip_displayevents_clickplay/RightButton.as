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
			trace("///this.play();");
			this.play();
			
			trace("///this.parent.l_button.play();");
			this.parent.l_button.play();
		}
	}
}