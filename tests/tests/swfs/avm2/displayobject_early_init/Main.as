package {
	import flash.display.MovieClip;

	public class Main extends MovieClip {
		// FIXME - add 'MyButton' as a timeline child once we fix
		// Ruffle's SimpleButton construction handling.
		public function Main() {
			trace("Constructing MyImage from ActionScript");
			var image = new MyImage(42, 24);
			trace("Image: " + image);

			new UnlinkedBitmapData();

			trace("Constructing MySprite from ActionScript");
			var mySprite = new MySprite();
			trace("mySprite: " + mySprite);

			trace("Constructing MyMovieClip from ActionScript");
			var myMovieClip = new MyMovieClip();
			trace("myMovieClip: " + myMovieClip);

			new UnlinkedSprite();
			new UnlinkedMovieClip();
			new UnlinkedLoader();
			new UnlinkedShape();
			new UnlinkedButton();
			new UnlinkedTextField();
			new UnlinkedByteArray();
			new UnlinkedSound();

			trace("Constructing MyButton from ActionScript");
			var myButton = new MyButton();
			trace("myButton: " + myButton);

			trace("Constructing BoundByteArray from ActionScript");
			var boundByteArray = new BoundByteArray();
			trace("boundByteArray: " + boundByteArray);
			
			trace("Constructing BoundSound from ActionScriot");
			var boundSound = new BoundSound();
			trace("boundSound: " + boundSound);
		}
	}
}