// SWF Version 7

/*
 * This test is the third test out of a series of tests, testing the result of getBounds / getRect when
 * called on a MovieClip with invalid bounds.
 * It can be either 6710886.35 or 6710886.4 for each corner of the rectangle.
 * Which of these it is depends on the SWF versions of the parent SWF files and previous getBounds / getRect
 * calls.
 * This needs to be tested in several tests because an internal state determining the result can change
 * irreversibly. Making sure that it changes correctly on different occasions takes several tests.
 *
 * Explanation of this test and the individual test files:
 * File 1 (Parent7L8.swf)
 * - getBounds should change to X.4 after the child calls it because the child called it with activation SWF
 * - version >= 8.
 * File 2 (Parent7L7.swf)
 * - getBounds should return X.4 in the parent and child file each because it has already been called and
 * - returned X.4.
 */

var loader = new MovieClipLoader();
loader.addListener(this);

trace("Test file 1");
var mc = createEmptyMovieClip("testMovieClip1", getNextHighestDepth());
loader.loadClip("Parent7L8.swf", mc);

/*
 * This function is called after the first test file has loaded.
 * It waits ten frames for the test file to complete and then starts loading the second test file.
 */
function onLoadInit(_) {
	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		if (frameCount == 10) {
			this.onEnterFrame = null;
			trace("\nTest file 2");
			var mc = createEmptyMovieClip("testMovieClip2", getNextHighestDepth());
			loadMovie("Parent7L7.swf", mc);
		}
	}
}
