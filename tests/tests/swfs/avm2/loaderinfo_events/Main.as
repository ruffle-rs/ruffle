	package {
		import flash.display.MovieClip;
		public class Main extends MovieClip {
			
			public function Main() {						
				loaderInfo.addEventListener("init", function() {
					trace("Called init!");
				});
				this.addEventListener("addedToStage", function() {
					trace("Called addedToStage");
				});
				this.addEventListener("added", function() {
					trace("Called added");
				});
			
				var clip = this;
				this.addEventListener("enterFrame", function() {
					trace("Called enterFrame");
				});
			
				this.addEventListener("exitFrame", function() {
					trace("Called exitFrame");
				});
			
				stage.loaderInfo.addEventListener("init", function() {
					trace("ERROR: Stage loaderInfo should not fire 'init'");
				});
				trace("Called constructor");
			}
		}
}