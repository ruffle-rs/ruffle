package  {
	
	import flash.display.MovieClip;
	
	
	public class PinkRect extends MovieClip {
		
		
		public function PinkRect() {
			trace('PinkRect.constructor');
			
			function listener(e) {
				trace('PinkRect.' + e.type);
			}
			
			addEventListener('enterFrame', listener);
			addEventListener('frameConstructed', listener);
			addEventListener('exitFrame', listener);
			
			addEventListener('addedToStage', function () {
				trace('PinkRect.addedToStage');
			});
			addEventListener('removedFromStage', function () {
				trace('PinkRect.removedFromStage');
							 
				removeEventListener('enterFrame', listener);
				removeEventListener('frameConstructed', listener);
				removeEventListener('exitFrame', listener);
			});
		}
	}
	
}
