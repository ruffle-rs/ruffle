package {
	import flash.media.Sound;

	[Embed(source="noise.mp3", mimeType="audio/mpeg")]
	public class BoundSound extends Sound {		
		public function BoundSound() {
			trace("BoundSound before super(): this.bytesLoaded = " + this.bytesLoaded + " this.bytesTotal = " + this.bytesTotal);
			super();
			trace("BoundSound after super(): this.bytesLoaded = " + this.bytesLoaded + " this.bytesTotal = " + this.bytesTotal);			
		}
	}
}