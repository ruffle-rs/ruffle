package {
	
	public class Test {
		public function Test() {
			trace(DEF); // NOTE: Modified to finddef
			trace(Test); // NOTE: Modified to finddef
			try {
				trace(aaa); // NOTE: Modified to finddef
			} catch (e) {
					trace(e);
			}
		}
	}
}

const DEF = "domain";