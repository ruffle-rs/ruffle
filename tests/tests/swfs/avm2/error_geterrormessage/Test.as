package {
	import flash.display.Sprite;

	public class Test extends Sprite {
		public function Test() {
			trace(Error.getErrorMessage(-1));
			trace(Error.getErrorMessage(0));
			trace(Error.getErrorMessage(1));
			trace(Error.getErrorMessage(42));
			trace(Error.getErrorMessage(100));
			// TODO:
			// Error #1000: The system is out of memory.
			// trace(Error.getErrorMessage(1000));
			// Error #1042: Not an ABC file.  major_version=%1 minor_version=%2.
			// trace(Error.getErrorMessage(1042));
			trace(Error.getErrorMessage(10000));
		}
	}
}