package {
	import flash.utils.getDefinitionByName;
	import flash.utils.getQualifiedClassName;
	
	public class Parent {
		public function Parent() {
			trace("Constructed Parent from child swf");
			trace("Roundtrip: " + getDefinitionByName(getQualifiedClassName(this)));
		}
	}
}