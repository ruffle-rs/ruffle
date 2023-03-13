package  {
	
	import flash.display.MovieClip;
	
	
	public class MainTimeline extends MovieClip {
		public static var frameScriptEvents:Array = [[], [], [], [], [], [], []];
		public static var done:Boolean = false;
		
		// These come from the timeline.
		
		// We don't add/remove this child from ActionScript - it will
		// get removed by a RemoveObject tag in the timeline, and will
		// stop running framescripts (it will *not* become an orphan)
		public var timelineChild:MyChild;
		
		// This comes from the timeline. We're going to swap this child with a manually created child -
		// it should not get removed at all
		public var toBeSwappedChild:MyChild;
		
		// These two timeline-created children will be swapped with each other
		public var otherTimeline1:MyChild;
		public var otherTimeline2:MyChild;
		
		public var removedTimelineChild:MyChild;
		
		// These come from ActionScript
		
		// We never add this orphan to the display list - it should
		// get framescripts run anyway, since it was created from ActionScript
		private var orphan:MyChild;
		
		// This is another child created from ActionScript. We're
		// going to swap it with a timeline-created child
		private var sneakyChild:MyChild;
		
		private var addedFromEnterFrame:MyChild;
		
		private var addedFromFrameScript:MyChild;
		
		public function MainTimeline() {
			
			var main = this;
			
			this.sneakyChild = new MyChild();
			this.sneakyChild.name = "SneakyChild";
			trace("Freshly constructed: this.sneakyChild.currentFrame = " + this.sneakyChild.currentFrame + " this.sneakyChild.isPlaying = " + this.sneakyChild.isPlaying);
					
			this.orphan = new MyChild();
			this.orphan.name = "Orphan";
			
			trace("toBeSwappedChild: " + this.toBeSwappedChild);
			this.addChild(this.sneakyChild);
			// Now that we've swapped a child that was placed by the timeline,
			// neither it nor the other swapped child (sneakyChild) should be removed
			// by the RemoveObject tag
			this.swapChildren(this.sneakyChild, this.toBeSwappedChild);
			
			// Swapping two timeline-created children with each other will
			// prevent them from getting removed
			this.swapChildren(this.otherTimeline1, this.otherTimeline2);
			
			// When we remove a child placed by the timeline, it
			// should be a regular orphan, and continue to tick.
			this.removeChild(this.removedTimelineChild);
			
			this.addEventListener("enterFrame", function() {
				if (MainTimeline.done) {
					return;
				}
			
				if (main.currentFrame == 2) {
					trace("Adding child in enterFrame");
					main.addedFromEnterFrame = new MyChild();
					main.addedFromEnterFrame.name = "addedFromEnterFrame";
					main.addChild(main.addedFromEnterFrame);
				}
			
				if (main.currentFrame == 3) {
					trace("Removing otherTimeline1");
					// This child had its depths swapped, so it should
					// be treated as a normal orphan (and continue running indefinitely)
					main.removeChild(main.otherTimeline1);
				}
			
				trace("MainTimeline frame: " + main.currentFrame);
				if (main.currentFrame == 8) {
					trace("Dummy currentFrame: " + new MyChild().currentFrame);
					trace("Main reached frame 8. Captured framescripts:");
					// Note - the relative order in which framescripts run is unpredictable -
					// it seems to be based on the iteration order of an internal hashtable.
					// To make our output reproducible in Ruffle, we have the framescript
					// for frame N append to the correspondid list in MainTimeline.frameScriptEvents
					// (each instance of MyChild will run a copy of the framescript).
					// We then sort each list and print them here.
					
					// Note - we don't print the last list. Due to the relative order
					// of framescripts and event listerns, it's difficult to run this
					// code in a place that will capture a all of the children that run,
					// without any from the 'next' frame. We include enough dummy frames at
					// the end that we capture any weird behavior that would happen
					// near adding/removing a child
					for (var i = 0; i < MainTimeline.frameScriptEvents.length - 1; i++) {
						trace("List entry " + i);
						MainTimeline.frameScriptEvents[i].sort();
						for each (var entry in MainTimeline.frameScriptEvents[i]) {
							trace(entry);
						}
					}
					trace("Done");
					MainTimeline.done = true;
					main.stop();
				}
			})			
		}
	}
	
}