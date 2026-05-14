package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	import flash.events.TextEvent;
	
	public class test extends MovieClip {
		public var text: TextField;
		
		public function test() {
			text.htmlText = "" +
				"<p>Click <a href='event:lower case'>here</a> to call a lower case event</p>" +
				"<p>Click <a href='Event:Mixed Case'>here</a> to call a mixed case event</p>" +
				"<p>Click <a href='EVENT:UPPER CASE'>here</a> to call a upper case event</p>" +
				"";
			text.addEventListener(TextEvent.LINK, this.onLink);
		}
		
		function onLink(event: TextEvent) {
			
			// Uncomment this line to get mouse coordinates for input.json
			// trace("Mouse event at X: " + stage.mouseX + " Y: " + stage.mouseY)
			
			trace("// event called");
			trace(event.text);
			trace("");
		}
	}
}