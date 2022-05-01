package {
	import flash.display.MovieClip;
	import flash.display.DisplayObject;
	import flash.display.DisplayObjectContainer;
	import flash.display.Shape;
	import flash.events.Event;
	
	public class MainTimeline extends EventWatcher {
		var invocation = 0;
		
		var destroy_me = false;
		
		public function MainTimeline() {
			super();
			this.addEventListener(Event.ENTER_FRAME, this.enter_frame_controller);
			this.addEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
		}
		
		function inspect(from: DisplayObject) {
			if (from instanceof DisplayObjectContainer) {
				var child, num_children = 0;
				
				// We don't want to test null children, so lie about the child count.
				for (var i = 0; i < from.numChildren; i += 1) {
					if (from.getChildAt(i)) {
						num_children += 1;
					}
				}
				
				trace("/// (Container:", from.name, "with", num_children, "children)");
				
				for (var i = 0; i < from.numChildren; i += 1) {
					child = from.getChildAt(i);
					if (child) {
						this.inspect(child);
					}
				}
			} else if (from instanceof Shape) {
				// Do nothing, since shapes will cause us to test the global instance count.
			} else {
				trace("/// (Unknown:", from.name, ")");
			}
		}
	
		function enter_frame_controller(evt: Event) {
			this.invocation++;
			
			if (this.invocation == 45) {
				this.destroy_me = true;
			}
			
			this.inspect(this);
		}
		
		function exit_frame_controller(evt: Event) {
			this.inspect(this);
			
			if (this.destroy_me) {
				this.destroy();
				this.l_button.destroy();
				this.m_button.destroy();
				this.r_button.destroy();
				
				this.removeEventListener(Event.ENTER_FRAME, this.enter_frame_controller);
				this.removeEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
				
				this.stop();
			}
		}
	}
}