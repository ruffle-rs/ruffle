package  {
	
	import flash.display.MovieClip;
	
	
	public class SuperClass extends MovieClip {
		
		public var target_from_superclass:TargetClip;
		
		public function SuperClass() {
			trace("SuperClass before super(): this.target_from_superclass=" + this.target_from_superclass);
			super();
			trace("SuperClass after  super(): this.target_from_superclass=" + this.target_from_superclass);
		}
	}
	
}
