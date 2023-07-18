package  {
	
	import flash.display.MovieClip;
	
	
	public class RedRect extends MovieClip {
		
		
		public function RedRect() {
			trace('RedRect.constructor');
			
			function listener(e) {
				trace('RedRect.' + e.type);
			}
			
			addEventListener('enterFrame', listener);
			addEventListener('frameConstructed', listener);
			addEventListener('exitFrame', listener);
			
			addEventListener('addedToStage', function () {
				trace('RedRect.addedToStage');
			});
			addEventListener('removedFromStage', function () {
				trace('RedRect.removedFromStage');
							 
				removeEventListener('enterFrame', listener);
				removeEventListener('frameConstructed', listener);
				removeEventListener('exitFrame', listener);
			});
			
			MovieClip(getChildAt(1)).gotoAndPlay(2);
			MovieClip(getChildAt(2)).gotoAndStop(2);
		}
	}
	
}
