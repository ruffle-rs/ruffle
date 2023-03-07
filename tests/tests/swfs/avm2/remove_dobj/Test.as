package {
	import flash.display.MovieClip;
	import flash.display.Sprite;
	
	public class Test {
		public function Test(main: MovieClip) {
			var first = new Sprite();
			var second = new Sprite();
			
			first.name = "First"
			second.name = "Second"
			
			first.mask = second;
			trace("Initial: first.mask: " + first.mask.name);
			
			main.addChild(first)
			trace("After add: first.mask: " + first.mask.name);
			
			main.removeChild(first)
			trace("After remove: first.mask: " + first.mask.name);
		}
	}
}