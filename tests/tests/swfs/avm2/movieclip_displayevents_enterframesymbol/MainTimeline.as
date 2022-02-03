package  {
	import flash.display.MovieClip;
	import flash.events.Event;
	import flash.system.System;
	
	public class MainTimeline extends EventWatcher {
		var invocation = 0;
		
		var destroy_me = false;
		
		var ew: EventWatcher = null;
		
		var ew2: EventWatcher = null;
		
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
			this.invocation++;
			
			switch (this.invocation) {
				default:
					break;
				case 1:
					trace("//this.ew = new EventWatcher();");
					this.ew = new EventWatcher();
					trace("//this.ew.name = \"ew\";");
					this.ew.name = "ew";

					trace("//this.addChild(this.ew);");
					this.addChild(this.ew);
					break;
				case 3:
					trace("//this.removeChild(this.ew);");
					this.removeChild(this.ew);

					trace("//this.ew.destroy();");
					this.ew.destroy();

					trace("//System.gc();");
					System.gc();
					break;
				case 4:
					trace("//this.ew = new EventWatcher();");
					this.ew = new EventWatcher();
					trace("//this.ew.name = 'parent';");
					this.ew.name = "parent";

					trace("//this.ew2 = new EventWatcher();");
					this.ew2 = new EventWatcher();
					trace("//this.ew2.name = 'child';");
					this.ew2.name = "child";

					trace("//this.ew.addChild(this.ew2);");
					this.ew.addChild(this.ew2);
					break;
				case 6:
					trace("//this.addChild(this.ew);");
					this.addChild(this.ew);
					break;
				case 7:
					trace("//this.ew.removeChild(this.ew2);");
					this.ew.removeChild(this.ew2);
					break;
				case 8:
					trace("//this.ew.addChild(this.ew2);");
					this.ew.addChild(this.ew2);
					break;
				case 9:
					trace("//this.removeChild(this.ew);");
					this.removeChild(this.ew);

					trace("//this.addChild(this.ew);");
					this.addChild(this.ew);

					trace("//this.addChild(this.ew2);");
					this.addChild(this.ew2);

					trace("//this.addChild(this.ew2);");
					this.addChild(this.ew2);

					trace("//this.ew.destroy();");
					this.ew.destroy();

					trace("//this.ew2.destroy();");
					this.ew2.destroy();

					trace("//System.gc();");
					System.gc();
					
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
				this.removeEventListener(Event.ENTER_FRAME, this.enter_frame_controller);
				this.removeEventListener(Event.EXIT_FRAME, this.exit_frame_controller);
			}
		}
	}
}
