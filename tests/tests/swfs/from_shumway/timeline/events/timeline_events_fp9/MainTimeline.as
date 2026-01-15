package  {
	
	import flash.display.MovieClip;
	
	
	public class MainTimeline extends MovieClip {
		
		
		public function MainTimeline() {
			trace('MainTimeline.constructor');
			
			function listener(e) {
				trace('MainTimeline.' + e.type);
			}
			
			addEventListener('enterFrame', listener);
			addEventListener('frameConstructed', listener);
			addEventListener('exitFrame', listener);
			
			addEventListener('addedToStage', function () {
				trace('MainTimeline.addedToStage');
			});
			
			addFrameScript(3, function () {
				stop();

				removeEventListener('enterFrame', listener);
				removeEventListener('frameConstructed', listener);
				removeEventListener('exitFrame', listener);
			});
		}
	}
	
}
