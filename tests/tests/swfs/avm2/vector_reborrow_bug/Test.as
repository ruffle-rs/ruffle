package {
	import flash.display.Sprite;

	public class Test extends Sprite {
		public function Test() {
			var v:Vector.<String> = new Vector.<String>();

			var i: int = 0;
			var sneaky:Object = {
			    toString: function():String {
			    	trace("toString called i = " + i);
			        v.join(","); // this will borrow the the vector again.
			        return "sneaky#" + (i++);
			    }
			};

			trace("before push: " + v);
			v.push(sneaky);

			trace("before unshift: " + v);
			v.unshift(sneaky);
			
			trace("before insertAt: " + v);
			v.insertAt(1, sneaky);

			trace("before splice: " + v);
			v.splice(0, 2, sneaky, sneaky);

			trace("done: " + v);
		}
	}
}