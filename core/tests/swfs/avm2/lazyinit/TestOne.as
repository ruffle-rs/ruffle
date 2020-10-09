package {
	public class TestOne {
		function TestOne() {
			trace("//TestOne constructor");
		}
	}
}

import TestTwo;

new TestTwo();

trace("//TestOne.as");