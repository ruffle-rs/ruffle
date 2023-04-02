package {
	import flash.display.DisplayObject;
	import flash.display.MovieClip;

	public class DuplicateClass extends MovieClip {
		public static const NAME:String = "DuplicateClass from loader_domain_other_child";
		
		public var childFromOtherDomain:DisplayObject;
		
		public function DuplicateClass() {
			trace("this.childFromOtherDomain = " + this.childFromOtherDomain);
		}
	}
	trace("loader_domain_other_child script initializer: SwfPrivateClass = " + SwfPrivateClass.NAME);
}