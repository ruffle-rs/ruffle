package {
	import flash.display.MovieClip;
	public class Test {
		public function Test() {
			tryCast("Hello", null);
			tryCast("Hello", undefined);
			tryCast("Hello", Class);
			tryCast(Object, null);
			tryCast(Object, undefined);
			tryCast(Object, Class);
			tryCast(null, null);
			tryCast(null, undefined);
			tryCast(undefined, null);
			tryCast(undefined, undefined);
		}
	
		private function tryCast(val: *, klass: Class) {
			try {
				trace(val + " as " + klass + ": " + (val as klass));
			} catch(e) {
				trace("Caught error from `" + val + " as " + klass + "`: " + e);
			}
		}
	}
}