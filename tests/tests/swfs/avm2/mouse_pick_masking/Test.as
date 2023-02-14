package {
	import flash.display.MovieClip;

	public class Test {
		public static function runTest(main:MovieClip) {
			main.stage.addEventListener("mouseDown", function(e) {
				trace("Click: " + e.target.name + " " + e.target + " stageX=" + e.stageX + " stageY=" + e.stageY);
			});
		}
	}
}