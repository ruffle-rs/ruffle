package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			// constructor code
		}
	}
	
}

import flash.utils.Dictionary;

function testNestedIter(obj: Object) {
	for (var i = 0; i < 100; i++) {
		obj["Key " + i] = i
	};

	var seen_main = [];
	var seen_sub = [];

	for (var key in obj) {
		seen_main.push("Key: '" + key + "' Value: " + obj[key]);

		if (seen_main.length == 50) {
			for (var key in obj) {
				seen_sub.push("Key: '" + key + "' Value: " + obj[key]);
			}
		}
	}

	seen_main.sort();
	seen_sub.sort();

	trace("Seen main: " + seen_main.length);
	trace("Seen sub: " + seen_sub.length);
	trace(seen_main);
	trace(seen_sub);
}

trace("Testing Object nested iteration");
testNestedIter(new Object());
trace()
trace("Testing Dictionary nested iteration");
testNestedIter(new Dictionary());
