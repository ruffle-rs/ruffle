package {
	import flash.utils.getDefinitionByName;
	public class Test {

		function throwAndCatch(e:*) {
			try {
				throw(e);
			} catch (err:ArgumentError) {
				trace("Caught ArgumentError: " + err);
			} catch (err:Error) {
				trace("Caught Error: " + err)
			} catch (err:RangeError) {
				// This should never run, since the more general 'err:Error'
				// block above catch any 'RangeError'
				trace("INCORRECT CATCH BLOCK: RangeError " + err);
			} catch (err:Object) {
				trace("Caught Object: " + err);
			} catch (err:*) {
				trace("Caught value: " + err);
			}
		}
		
		public function Test() {
			throwAndCatch(new ArgumentError("My argument error"));
			throwAndCatch(new Error("My error"));
			throwAndCatch(new RangeError("My range error"));
			throwAndCatch(new Object());
			throwAndCatch(true);
			throwAndCatch(null);
			throwAndCatch(undefined);
			
			try {
				getDefinitionByName("some.package.FakeClass");
			} catch (err:Error) {
				trace("getDefinitionByName threw: " + err);
				trace(err.getStackTrace());
			}
		}
	}
}