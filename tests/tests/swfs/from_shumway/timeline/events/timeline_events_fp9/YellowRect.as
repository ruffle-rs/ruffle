package  {
	
	import flash.display.MovieClip;
	
	
	public class YellowRect extends MovieClip {
		
		
		public function YellowRect() {
			trace('YellowRect.constructor');
			
			function listener(e) {
				trace('YellowRect.' + e.type);
			}
			
			addEventListener('enterFrame', listener);
			addEventListener('frameConstructed', listener);
			addEventListener('exitFrame', listener);
			
			addEventListener('addedToStage', function () {
				trace('YellowRect.addedToStage');
			});
			addEventListener('removedFromStage', function () {
				trace('YellowRect.removedFromStage');
							 
				removeEventListener('enterFrame', listener);
				removeEventListener('frameConstructed', listener);
				removeEventListener('exitFrame', listener);
			});
		}
	}
	
}
