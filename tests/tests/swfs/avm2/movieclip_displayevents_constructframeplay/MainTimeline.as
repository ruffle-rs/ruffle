package {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class MainTimeline extends EventWatcher {
		var invocation = 0;
		
		var destroy_me = false;
		
		public function MainTimeline() {
			super();
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.construct_frame_controller);
			this.addEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
		}
		
		function inspect() {
			var children = "", child;
			
			for (var i = 0; i < this.numChildren; i += 1) {
				child = this.getChildAt(i);
				if (child) {
					children += child.name + " ";
				}
			}
		
			trace("///Children:", children);
		}
	
		function construct_frame_controller(evt: Event) {
			switch (this.invocation) {
				case 0:
				default:
					trace("/// (Stopping root clip in frameConstructed...)");
					this.stop();
					break;
				case 1:
				case 3:
				case 4:
					trace("/// (Playing root clip in frameConstructed...)");
					this.play();
					break;
				case 5:
					trace("/// (Stopping root clip in frameConstructed and flagging event handler for removal...)");
					this.stop();
					this.destroy_me = true;
					break;
			}
			
			this.invocation++;
			this.inspect();
		}
		
		function exit_frame_controller(evt: Event) {
			this.inspect();
			
			if (this.destroy_me) {
				this.stop();
				this.destroy();
				this.removeEventListener(Event.FRAME_CONSTRUCTED, this.construct_frame_controller);
				this.removeEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
			}
		}
	}
}