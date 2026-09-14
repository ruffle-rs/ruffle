package {
	import flash.display.MovieClip;
	import flash.utils.Dictionary;

	public class Test extends MovieClip {

		public function Test() {
			var keys = [];
			trace("Using dictionary");
			var dict = new Dictionary();
			dict[1] = "1-int";
			dict[2] = "2-int";
			dict[-3] = "int: Negative 3";
			dict[2.5] = "2.5-float";
			dict[true] = "true-bool";
			dict[false] = "false-bool";
			dict[268435455] = "2^28-1-number";
			dict[268435456] = "2^28-number";
			dict[2147483646] = "intmax-1-number";
			dict[2147483647] = "intmax-number";
			dict[4294967294] = "uintmax-1-number";
			dict[4294967295] = "uintmax-number";
			
			this.printKeys(dict);
		
			dict["1"] = "1-string";
			dict["2.5"] = "2.5-string"
			dict["-3"] = "String: Negative 3";
			dict["true"] = "true-string";
			dict["268435455"] = "2^28-1-string";
			dict["268435456"] = "2^28-string";
			dict["2147483647"] = "intmax-1-string";
			dict["2147483648"] = "intmax-string";
			dict["4294967294"] = "uintmax-1-string";
			dict["4294967295"] = "uintmax-string";
			trace("After update:");
		
			this.printKeys(dict);
		
			trace();
			trace("Using Object");
			var obj = new Object();
			obj[1] = "1-int";
			obj[2] = "2-int";
			obj[-3] = "int: Negative 3";
			obj[2.5] = "2.5-float"
			obj[true] = "true-bool";
			obj[false] = "false-bool";
			obj[268435455] = "2^28-1-number";
			obj[268435456] = "2^28-number";
			obj[2147483646] = "intmax-1-number";
			obj[2147483647] = "intmax-number";
			obj[4294967294] = "uintmax-1-number";
			obj[4294967295] = "uintmax-number";
			
			this.printKeys(obj);
			

			obj["1"] = "1-string";
			obj["2.5"] = "2.5-string";
			obj["-3"] = "String: Negative 3"
			obj["true"] = "true-string";
			obj["268435455"] = "2^28-1-string";
			obj["268435456"] = "2^28-string";
			obj["2147483646"] = "intmax-1-string";
			obj["2147483647"] = "intmax-string";
			obj["4294967294"] = "uintmax-1-string";
			obj["4294967295"] = "uintmax-string";
			trace("After update:");
			
			this.printKeys(obj);
		}

		private function printKeys(obj: Object) {
			var keys = [];
			for (var key in obj) {
				keys.push(key);
			}
			keys.sort();

			for each (var key in keys) {
				trace("key = " + key + " (typeof key = " + typeof key + ") val = " + obj[key]);
			}
		}
	}
}
