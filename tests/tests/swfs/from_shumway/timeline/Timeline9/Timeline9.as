package  {
	
	import flash.display.MovieClip;
	import flash.utils.setTimeout;
	
	public class Timeline9 extends MovieClip {
		
		
		public function Timeline9() {
			setTimeout(function () { trace('1->2'); nextFrame(); }, 100);
			setTimeout(function () { trace('2->3'); nextFrame(); }, 200);
			setTimeout(function () { trace('3->?'); nextFrame(); }, 300);
			setTimeout(function () { trace('3->2'); prevFrame(); }, 400);
			setTimeout(function () { trace('2->1'); prevFrame(); }, 500);
			setTimeout(function () { trace('1->?'); prevFrame(); }, 600);
		}
	}
	
}
