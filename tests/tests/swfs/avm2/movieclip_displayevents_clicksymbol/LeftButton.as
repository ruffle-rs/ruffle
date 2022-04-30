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
			
			if (this.parent.left_shark === null) {
				var ew = new EventWatcher();
				ew.name = "left_shark";
				
				this.parent.addChild(ew);
				this.parent.left_shark = ew;
			}
		}
	}
}