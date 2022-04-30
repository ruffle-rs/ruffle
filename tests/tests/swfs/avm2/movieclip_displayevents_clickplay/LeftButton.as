package {
	import flash.events.MouseEvent;
	import flash.events.Event;
	
	public class LeftButton extends EventWatcher {
		public function LeftButton() {
			super();
			
			this.addEventListener(MouseEvent.CLICK, this.clicked);
			this.stop();
		}
	
		public function clicked(event: Event) {
			trace("///(LeftButton clicked...)");
			trace("///this.stop();");
			this.stop();
			
			trace("///this.parent.r_button.stop();");
			this.parent.r_button.stop();
		}
	}
}