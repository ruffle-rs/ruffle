package
{
	import flash.display.Sprite;
	import flash.events.Event;
	import flash.media.Sound;
	import flash.net.URLRequest;
	
	public class Main extends Sprite
	{
		
		public function Main()
		{
			var sound:Sound = new Sound();
			sound.addEventListener(Event.ID3, function(event:Event):void{
				trace("id3event id3:" + sound.id3);
				var properties:Array = ["album", "artist", "comment", "genre", "songName", "track", "year"];
				for (var i:String in properties) trace(properties[i]+":"+sound.id3[properties[i]]);
			});
			sound.load(new URLRequest("test_audio.mp3"));
		}
	
	}

}
