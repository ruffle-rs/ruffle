package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			trace("XML settings: " + XML[new QName("", "settings")]);
			trace("XML setSettings: " + XML[new QName("", "setSettings")]);
			trace("XML defaultSettings: " + XML[new QName("", "defaultSettings")]);
		}

	}
	
}
