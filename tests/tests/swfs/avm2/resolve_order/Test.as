package {
	
	public class Test {
		public function Test() {
			
		}
	}
}

this["int"] = "global";
this["other"] = "global";
trace(int);
trace(other);

function test() {
	trace(int);
	trace(other);
}

test();