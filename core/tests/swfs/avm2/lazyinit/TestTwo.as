package {
	public class TestTwo {
		{
			trace("//TestTwo class init");
		}
		
		function TestTwo() {
			trace("//TestTwo constructor");
		}
	}
}

trace("//TestTwo.as start");

import TestOne;

new TestOne();

trace("//TestTwo.as end");