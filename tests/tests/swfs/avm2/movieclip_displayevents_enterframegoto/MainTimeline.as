package {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class MainTimeline extends EventWatcher {
		var invocation = 0;
		
		var destroy_me = false;
		
		public function MainTimeline() {
			super();
			this.addEventListener(Event.ENTER_FRAME, this.enter_frame_controller);
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
	
		function enter_frame_controller(evt: Event) {
			switch (this.invocation) {
				default:
					break;
				case 2:
					trace("/// (gotoAndStop(3) in enterFrame...)");
					this.gotoAndStop(3);
					break;
				case 4:
					trace("/// (gotoAndPlay(1) in enterFrame...)");
					this.gotoAndPlay(1);
					break;
				case 7:
					trace("/// (gotoAndPlay(3) in enterFrame...)");
					this.gotoAndPlay(3);
					break;
				case 10:
					trace("/// (gotoAndStop(2) in enterFrame...)");
					this.gotoAndStop(2);
					break;
				case 12:
					trace("/// (gotoAndPlay(1) in enterFrame...)");
					this.gotoAndPlay(1);
					break;
				case 13:
					trace("/// (gotoAndPlay(3) in enterFrame...)");
					this.gotoAndPlay(3);
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
				this.removeEventListener(Event.ENTER_FRAME, this.enter_frame_controller);
				this.removeEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
			}
		}
	}
}