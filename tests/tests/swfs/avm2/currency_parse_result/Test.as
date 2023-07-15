package {
	import flash.display.MovieClip;
	import flash.globalization.CurrencyParseResult;
	public class Test extends MovieClip {
		public function Test() {
			var cr1:CurrencyParseResult = new CurrencyParseResult(NaN, "");
			trace(cr1.currencyString);
			trace(cr1.value);
			var cr2:CurrencyParseResult = new CurrencyParseResult(10, "$");
			trace(cr2.currencyString);
			trace(cr2.value);
			var cr3:CurrencyParseResult = new CurrencyParseResult(200.10, "£");
			trace(cr3.currencyString);
			trace(cr3.value);
			var cr4:CurrencyParseResult = new CurrencyParseResult(-10, "€");
			trace(cr4.currencyString);
			trace(cr4.value);
		}
	}
}
