package {
	import flash.display.MovieClip;
	import flash.display.DisplayObject;

	public class DuplicateClass extends MovieClip {
		public var childFromDomainChild: DisplayObject;
		public function DuplicateClass() {
			trace("this.childFromDomainChild = " + this.childFromDomainChild);
			trace("Child name: " + this.getChildAt(0));
		}
		public static const NAME:String = "DuplicateClass from loader_domain_child";
	}
	trace("loader_domain_child script initializer: SwfPrivateClass = " + SwfPrivateClass.NAME);
}