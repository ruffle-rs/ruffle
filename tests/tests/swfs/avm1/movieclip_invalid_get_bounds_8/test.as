// SWF Version 7

/*
 * This test is the eighth test out of a series of tests, testing the result of getBounds / getRect when
 * called on a MovieClip with invalid bounds.
 * It can be either 6710886.35 or 6710886.4 for each corner of the rectangle.
 * Which of these it is depends on the SWF versions of the parent SWF files and previous getBounds / getRect
 * calls.
 * This needs to be tested in several tests because an internal state determining the result can change
 * irreversibly. Making sure that it changes correctly on different occasions takes several tests.
 *
 * Explanation of this test and the individual test files:
 * File 1 (8LTrace8.swf)
 * - The trace statements should be printed; no getBounds statement exists.
 * File 2 (Parent7L7.swf)
 * - getBounds should return X.35 in the parent and child file each because the activation SWF version and
 * - all parent SWF versions are < 8.
 * - The other loaded SWF files with SWF version >= 8 should not influence this.
 */

var loader = new MovieClipLoader();
loader.addListener(this);

trace("Test file 1");
var mc = createEmptyMovieClip("testMovieClip1", getNextHighestDepth());
loader.loadClip("8LTrace8.swf", mc);

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
