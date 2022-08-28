package {
	import flash.net.SharedObject;
	public class Test {

		static function storeData(data: Object) {
			
			var sparse = new Array(5);
			sparse[0] = "elem0";
			sparse[4] = "elem4";
			sparse[-1] = "elem negative one";
			
							
			var dense = new Array(3);
			dense[0] = 1;
			dense[1] = 2;
			dense[2] = 3;
			
			// Store everything in an array to work around Ruffle's incorrect
			// object property serialization order
			data.props = [
				true,
				"hello",
				"something else",
				[uint(10), uint((1 << 28) - 2), uint((1 << 28) - 1), uint((1 << 28)), uint((1 << 28) + 1), uint((1 << 28) + 2)],
				[int(10), int((1 << 28) - 2), int((1 << 28) - 1), int((1 << 28)), int((1 << 28) + 1), int((1 << 28) + 2),
					int(-(1 << 28) - 2), int(-(1 << 28) - 1), int(-(1 << 28)), int(-(1 << 28) + 1), int(-(1 << 28) + 2)],
				-5.1,
				sparse,
				dense,
				new Date(2147483647),
				// FIXME - enable this when Ruffle fully implements AVM2 XML
				// new XML("<test>Test</test>")
			];
			data.hidden = "Some hidden value";
			data.setPropertyIsEnumerable("hidden", false);			
		}
		
		public static function main() {
		
			// The serialization order for object keys in Flash player depends
			// on the enumeration order of the 'data' object, which in turn
			// depends on the placement of objects on the heap. This appears to
			// be deterministic between runs of flash player, but the effect of
			// adding or removing a property has an unpredictable effect on the order.
			// Since Ruffle doesn't implement Flash's hash-based enumeration order,
			// the AMF '.sol' file we write out may have object properties written
			// in a different order (though it should still deserialize to the same object).
			//
			// To work around this, we create two SharedObjects
			// 1. RuffleTest only stores an array of simple objects, so the AMF output should match exactly between Ruffle and Flash Player
			// 2. RuffleTestNonDeterministic stores objects with several properties, so we don't compare the AMF (but we do print the deserialized object)
			var obj = SharedObject.getLocal("RuffleTest", "/");
			var otherObj = SharedObject.getLocal("RuffleTestNonDeterministic", "/")
			
			trace("typeof obj =" + typeof obj.data);
			
			trace("typeof otherObj =" + typeof otherObj.data);
		
			if(obj.data.props === undefined) {
				trace("No data found. Initializing...");
				
				storeData(obj.data)
				storeData(otherObj.data)
				
				//Only set this on the object that we *don't* compare with flash,
				//since we don't yet match the object property serialization order correctly.
				otherObj.data.o = {a: "a", b: "b"};
			
				trace("otherObj.data.props:")
				dump(otherObj.data.props);
			
				obj.flush();
				otherObj.flush();
			} else {		
				trace("obj.data.hidden: " + obj.data.hidden);
				trace("otherObj.data.hidden: " + otherObj.data.hidden);
				
				trace()
				trace("obj dump:")
				dump(obj.data.props);
				
				trace()
				trace("otherObj dump:")
				dump(otherObj.data.props)
			}
		}
	}	
}

function dump(obj:Object) {
	var keys = [];
	for (var key in obj) {
		keys.push(key);
	}
	keys.sort();
	for (var i in keys) {
		var k = keys[i];
		var val = obj[k];
		if (val instanceof Date) {
			// Printing 'val.toString()' depends on the system time zone,
			// so use UTC to make the output reproducible 
			trace(k, "= (UTC) ", val.toUTCString());
		} else {
			trace(k, "=", val.toString(), "type", typeof val);
		}

	}
}
