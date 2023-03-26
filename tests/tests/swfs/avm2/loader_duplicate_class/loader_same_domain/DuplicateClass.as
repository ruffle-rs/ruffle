package {
	import flash.display.DisplayObject;
	import flash.display.MovieClip;

	public class DuplicateClass extends MovieClip {
		public static const NAME:String = "DuplicateClass from loader_same_domain";
		
		public var childFromSameDomain:DisplayObject;
		public var childFromOtherDomain:DisplayObject;
		public var childFromDomainChild:DisplayObject;
		
		public function DuplicateClass() {
			trace("loader_same_domain DuplicateClass: this.childFromSameDomain = " + this.childFromSameDomain + " this.childFromOtherDomain = " + this.childFromOtherDomain + " this.childFromDomainChild = " + this.childFromDomainChild);
		}
	}

	trace("loader_same_domain DuplicateClass script initializer: SwfPrivateClass = " + SwfPrivateClass.NAME);
}