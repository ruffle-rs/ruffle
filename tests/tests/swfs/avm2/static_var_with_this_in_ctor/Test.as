package  {
	
	
	public class Test {
		public static var INSTANCE: Test = resolve_param(new Test(null));
		
		public function Test(prev: Test = null) {
			trace("Test(" + prev + ")");
		}
	
		private static function resolve_param(param:Test):Test {
			return param;
		}
	}
	
}
