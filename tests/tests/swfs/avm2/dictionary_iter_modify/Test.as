package {
	import flash.utils.Dictionary;

	public class Test {
		public function Test() {
			runTest(new Object());
			
			var array = new Array();
			array.push("First normal entry");
			array.push("Second normal entry");
			//runTest(array);
			
			var dict = new Dictionary();
			//dict[new Object()] = "First distinct object key";
			//dict[new Object()] = "Second distinct object key";
			runTest(dict);
		}
			
		private function runTest(obj: Object) {
			trace("Initial object: " + obj);
			for (var i = 0; i < 100; i++) {
				obj["Key " + i] = i
			};

			var toDelete = [];
		
			var seen = [];
			for (var key in obj) {
				seen.push("Key: '" + key + "' Value: " + obj[key]);
				
				if (seen.length < 94) {
					toDelete.push(key);
				}
				
				if (seen.length == 95) {
					for (var j = 0; j < toDelete.length; j++) {
						var newKey = toDelete[j];
						if (j % 2 == 0) {
							delete obj[newKey];
						} else {
							//obj.setPropertyIsEnumerable(newKey, false);
						}
					}					
				}
			}
			// The order of entries pushed to 'seen' depends on the internal
			// hash table iteration order, so sort the output to make it comparable
			// between Flash Player and Ruffle
			seen.sort();
			trace("Seen during iteration:");
			trace(seen);
		
			var seenAfter = [];
			for (var newKey in obj) {
				seenAfter.push("Key: '" + newKey + "' Value: " + obj[newKey]);
			}
			seenAfter.sort();
			// We delete keys after seeing them during iteration. Since
			// the iteration order will be different between Flash Player and Ruffle,
			// we'll delete different keys. Therefore, we only check the *number* of
			// keys remaining.
			trace("Seen during iteration after deletion: " + seenAfter.length);
		}
	}
}