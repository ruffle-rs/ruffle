package {
	import flash.utils.getDefinitionByName;
	public class Test {
		public static const MY_CONST:String = "foo";
		public function Test() {
			var obj = getDefinitionByName("Test");
			trace("MY_CONST: " + obj.hasOwnProperty("MY_CONST"));
			trace("MISSING_CONST: " + obj.hasOwnProperty("MISSING_CONST"));
		}
	}
}