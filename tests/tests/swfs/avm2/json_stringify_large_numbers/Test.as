package {
	import flash.display.Sprite;

	public class Test extends Sprite {
		public function Test() {
			// Numbers are written like String(n). Whole numbers outside -2^28 .. 2^28-1
			// are stored as Number, but still have no fractional part.
			trace(JSON.stringify([268435455, 268435456]));
			trace(JSON.stringify([-268435456, -268435457]));
			trace(JSON.stringify([2147483647, 2147483648, 4294967296]));
			trace(JSON.stringify({value: 1000000000}));
			trace(JSON.stringify([NaN, Infinity, -Infinity, -0, 1.5, 9007199254740992, -9007199254740992, 1e18]));
			trace(JSON.stringify([9007199254740994, 123456789012345680, 9223372036854774784, 9223372036854775808, -9223372036854775808, -9223372036854777856, 1e19, 1e20, 1e21]));
			trace(JSON.stringify([0.1, 0.000001, 1e-7, 123.456, -1.75e12, 1.5e300, 0.1 + 0.2]));
			trace(JSON.stringify({a: [268435456, 1.5]}, null, 2));
		}
	}
}
