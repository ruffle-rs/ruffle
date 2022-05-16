package  {
	import flash.display.Sprite;
	public class Bench extends Sprite {
		public static function bench(): uint {
			return fib(20);
		}

		public static function fib(n: uint): uint {
			if( n <= 1 ) {
				return n;
			} else {
				return fib(n-2) + fib(n-1);
			}
		}
	}
}
