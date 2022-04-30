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
			
			if (this.parent.left_shark) {
				this.parent.removeChild(this.parent.left_shark);
				this.parent.left_shark.destroy();
				this.parent.left_shark = null;
			}
		}
	}
}