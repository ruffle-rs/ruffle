package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	import flash.events.TextEvent;
	
	
	public class Test extends MovieClip {
		public var text: TextField;
		
		public function Test() {
			text.htmlText = "" +
				"<p>Click <a href='event:alert1'>here</a> to call alert 1</p>" +
				"<p>Or click <a href='event:Second Test'>here</a> to call alert 2</p>" +
				"<p><a href='event:'>this one is empty</a></p>" +
				"<p>But <a href='event:a,b,c,d'>this one</a> has lots of args!</p>" +
				"";
			text.addEventListener(TextEvent.LINK, this.onLink);
		}
		
		function onLink(event: TextEvent) {
			trace("/// onLink called!");
			trace("// event.text");
			trace(event.text);
			trace("");
			
			trace("// event.bubbles");
			trace(event.bubbles);
			trace("");
			
			trace("// event.cancelable");
			trace(event.cancelable);
			trace("");
			
			trace("// event.currentTarget");
			trace(event.currentTarget);
			trace("");
			
			trace("// event.target");
			trace(event.target);
			trace("");
			
			trace("");
		}
	}
	
}
