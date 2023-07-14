package {
	import flash.display.MovieClip;
	import flash.display.DisplayObject;
	import flash.events.Event;
	import flash.display.Sprite;

	public class Main extends MovieClip {
		public var myChild:DisplayObject;
		public var manualRemoval:DisplayObject;
		public var reAdded:DisplayObject;
		
		public function get nullGetSet():DisplayObject {
			trace("Invoking nullGetSet getter");
			return null;
		}
	
		public function set nullGetSet(val:DisplayObject):void {
			trace("Invoking nullGetSet setter with " + val);
		}
	
		public function get undefinedGetSet():DisplayObject {
			trace("Invoking undefinedGetSet getter");
			return undefined;
		}
	
		public function set undefinedGetSet(val:DisplayObject):void {
			trace("Invoking undefinedGetSet setter with " + val);
		}
	
		public function get normalGetSet() {
			trace("Invoking normalGetSet getter");
			return new Sprite();
		}
	
		public function set normalGetSet(val: DisplayObject) {
			trace("Invoking normalGetSet setter with " + val);
		}
	
		public function get exceptionGetSet(): DisplayObject {
			trace("Invoking exceptionGetSet");
			throw new Error("Called exceptionGetSet getter"); 
		}
	
		public function set exceptionGetSet(val: DisplayObject):void {
			trace("Invoking exceptionGetSet setter with " + val);
		}
		
		public function Main() {
			this.addEventListener(Event.REMOVED, this.onRemoved);
		}
	
		private function onRemoved(e: Event) {
			trace("Event.REMOVED: " + e);
			trace("Event.REMOVED: e.target = " + e.target + " e.target.name = " + e.target.name);
			trace("Event.REMOVED: this.myChild=" + this.myChild + " this.manualRemoval=" + manualRemoval + " this.reAdded=" + this.reAdded);
		}
	}
}