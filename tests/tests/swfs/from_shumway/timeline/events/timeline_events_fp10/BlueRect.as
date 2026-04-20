package  {
	
	import flash.display.MovieClip;
	
	
	public class BlueRect extends MovieClip {
		
		
		public function BlueRect() {
			trace('BlueRect.constructor');
			
			function listener(e) {
				trace('BlueRect.' + e.type);
			}
			
			addEventListener('enterFrame', listener);
			addEventListener('frameConstructed', listener);
			addEventListener('exitFrame', listener);
			
			addEventListener('addedToStage', function () {
				trace('BlueRect.addedToStage');
			});
			addEventListener('removedFromStage', function () {
				trace('BlueRect.removedFromStage');
							 
				removeEventListener('enterFrame', listener);
				removeEventListener('frameConstructed', listener);
				removeEventListener('exitFrame', listener);
			});
		}
	}
	
}
