package  {
	
	import flash.display.MovieClip;
    import flash.media.Sound;
    import flash.media.SoundChannel;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
	        var snd:Sound = new Sound();
	        var channel:SoundChannel = new SoundChannel();

	        trace("// channel");
	        trace(channel);
	        trace("");

	        trace("// channel.soundTransform");
	        trace(channel.soundTransform);
	        trace("");

		}
	}
	
}
