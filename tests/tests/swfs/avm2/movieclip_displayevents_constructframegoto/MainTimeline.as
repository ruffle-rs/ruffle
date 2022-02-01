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
			this.invocation++;
			
			switch (this.invocation) {
				default:
					break;
				case 3:
					trace("/// (gotoAndStop(3) in frameConstructed...)");
					this.gotoAndStop(3);
					break;
				case 5:
					trace("/// (gotoAndPlay(1) in frameConstructed...)");
					this.gotoAndPlay(1);
					break;
				case 9:
					trace("/// (gotoAndPlay(3) in frameConstructed...)");
					this.gotoAndPlay(3);
					break;
				case 13:
					trace("/// (gotoAndStop(2) in frameConstructed...)");
					this.gotoAndStop(2);
					break;
				case 15:
					trace("/// (gotoAndPlay(1) in frameConstructed...)");
					this.gotoAndPlay(1);
					break;
				case 17:
					trace("/// (gotoAndPlay(3) in frameConstructed...)");
					this.gotoAndPlay(3);
					this.destroy_me = true;
					break;
			}
			
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