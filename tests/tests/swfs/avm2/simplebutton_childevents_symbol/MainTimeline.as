package  {
	import flash.display.MovieClip;
	import flash.display.DisplayObject;
	import flash.display.DisplayObjectContainer;
	import flash.system.System;
	
	public class MainTimeline extends EventWatcher {
		public function MainTimeline() {
			trace("//Constructed MainTimeline!");
			
			this.my_button.setup();
		}
		
		public function stop_display_object_handlers(dobj: DisplayObject) {
			if (dobj instanceof ButtonEventWatcher) {
				dobj.destroy();
				
				if (dobj.upState) {
					stop_display_object_handlers(dobj.upState);
				}
				
				if (dobj.downState) {
					stop_display_object_handlers(dobj.downState);
				}
				
				if (dobj.overState) {
					stop_display_object_handlers(dobj.overState);
				}
				
				if (dobj.hitTestState) {
					stop_display_object_handlers(dobj.hitTestState);
				}
			}
			
			if (dobj instanceof EventWatcher) {
				dobj.destroy();
			}
			
			if (dobj instanceof DisplayObjectContainer) {
				for (var i = 0; i < dobj.numChildren; i += 1) {
					stop_display_object_handlers(dobj.getChildAt(i));
				}
			}
		}
	}
	
}
