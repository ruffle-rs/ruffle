package {
	public class TestOne {
		{
			trace("//TestOne class init");
		}
		
		function TestOne() {
			trace("//TestOne constructor");
		}
	}
}

trace("//TestOne.as start");

import TestTwo;

new TestTwo();

trace("//TestOne.as end");