// SWF Version 7

/*
 * This test is the first test out of a series of tests, testing the result of getBounds / getRect when
 * called on a MovieClip with invalid bounds.
 * It can be either 6710886.35 or 6710886.4 for each corner of the rectangle.
 * Which of these it is depends on the SWF versions of the parent SWF files and previous getBounds / getRect
 * calls.
 * This needs to be tested in several tests because an internal state determining the result can change
 * irreversibly. Making sure that it changes correctly on different occasions takes several tests.
 *
 * Explanation of this test:
 * - In this test, the oldest parent file's SWF version is < 8. Additionally, every call of getBounds
 * - is with the activation SWF version < 8.
 * - Therefore, every getBounds call should return X.35, no matter whether and how many SWF files with
 * - SWF version >= 8 are loaded between the original file / the oldest parent file and the file calling
 * - getBounds.
 * - This test tests different constructs of files loading files with different SWF versions to make sure
 * - that Ruffle behaves correctly, no matter which files with which versions are loaded in between.
 */

var testFiles = ["Child7.swf", "Parent7L7.swf", "7LParent7L7.swf", "8L7LParent7L7.swf", "8LParent7L7.swf",
	"7L8LParent7L7.swf", "8L8LParent7L7.swf", "SP8L7.swf", "7LSP8L7.swf", "8LSP8L7.swf", "8L7LSP8L7.swf"];

var loader = new MovieClipLoader();
loader.addListener(this);

var currentTest = 0;

loadNextTest();

/*
 * This function is called after each test file and loads the next test file.
 */
function loadNextTest() {
	currentTest++;
	if (currentTest <= 0 || currentTest > testFiles.length + 1) {
		trace("\nError: Test file number " + currentTest + " does not exist.");
		return;
	} else if (currentTest == testFiles.length + 1) {
		trace("\nFinished all test files.");
		return;
	} else if (currentTest != 1) {
		trace("");
	}

	trace("Test file " + currentTest);

	var mc = createEmptyMovieClip("testMovieClip" + currentTest, getNextHighestDepth());
	loader.loadClip(testFiles[currentTest - 1], mc);
}

/*
 * This function is called after each test file has loaded.
 * It waits ten frames for the test file to complete and then starts loading the next test file.
 */
function onLoadInit(_) {
	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		if (frameCount == 10) {
			this.onEnterFrame = null;
			loadNextTest();
		}
	}
}
