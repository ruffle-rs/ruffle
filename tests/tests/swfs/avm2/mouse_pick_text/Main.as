package  {
	
	import flash.display.MovieClip;
	import flash.display.Sprite;
	import flash.events.MouseEvent;
	import flash.text.TextField;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			super();
			this.stage.addEventListener("mouseDown", function(e) {
				trace("mouseDown: " + e.target + " at: " + e.stageX + " " + e.stageY);
			});
			var childClip = Sprite(this.getChildAt(0));
		
			var firstTimelineText = childClip.getChildAt(0);
			// Uncomment this line to verify that the coordinates in our input.json
			// are correct (a new event should get generated)
			//firstTimelineText.mouseEnabled = false;
		
			var secondTimelineText = childClip.getChildAt(1);
			trace("Setting mouseEnabled=false for: " + secondTimelineText.text);
			trace("Before: secondTimelineText.selectable = " + secondTimelineText.selectable);
			secondTimelineText.mouseEnabled = false;
			trace("After: secondTimelineText.selectable = " + secondTimelineText.selectable);
				
			// The timeline-add field remembers that it originally came from the timeline
			//childClip.removeChild(firstTimelineText)
			//childClip.addChildAt(firstTimelineText, 0)
		
		
			var avmText = new TextField();
			avmText.y = 100;
			avmText.text = "Hello from AS3";
			
			var avmTextNonSelectable = new TextField();
			avmTextNonSelectable.selectable = false;
			avmTextNonSelectable.x = 140;
			avmTextNonSelectable.y = 100;
			avmTextNonSelectable.text = "AS3 non-selectable";
		
			childClip.addChild(avmText);
			childClip.addChild(avmTextNonSelectable);
		}
	}
	
}
