package {

import flash.display.Sprite;

public class Test extends Sprite {
	public function Test() {
		traceError(Error.getErrorMessage(-1000000));
		traceError(Error.getErrorMessage(-100));
		traceError(Error.getErrorMessage(-1));
		traceError(Error.getErrorMessage(0));
		traceError(Error.getErrorMessage(20));
		traceError(Error.getErrorMessage(100));

		for (var i = 999; i < 1135; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 1498; i < 1523; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 1998; i < 2208; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 2498; i < 2505; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 2998; i < 3018; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3098; i < 3145; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3198; i < 3232; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3299; i < 3372; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3399; i < 3404; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3499; i < 3504; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		for (var i = 3599; i < 3809; ++i) {
			traceError(Error.getErrorMessage(i));
		}

		traceError(Error.getErrorMessage(10000));
	}

	private function traceError(err:*):void {
		trace(err + ";");
	}
}

}
