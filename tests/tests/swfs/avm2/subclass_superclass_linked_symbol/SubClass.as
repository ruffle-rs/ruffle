package  {
	
	import flash.display.MovieClip;
	
	
	public class SubClass extends SuperClass {
		
		public var target_from_subclass:TargetClip;
		
		public function SubClass() {
			trace("SubClass before super(): this.target_from_subclass = " + this.target_from_subclass + " this.target_from_superclass=" + this.target_from_superclass);
			super();
			trace("SubClass after  super(): this.target_from_subclass = " + this.target_from_subclass + " this.target_from_superclass=" + this.target_from_superclass);
		}
	}
	
}
