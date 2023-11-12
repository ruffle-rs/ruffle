// SWF Version 10

/*
 * This test tests the different states of a MovieClip.
 * A state of a MovieClip consists of the values of all properties and the results of some getter
 * functions of the MovieClip.
 *
 * The states a MovieClip can be in are the following:
 * - Default State
 *     This is the default state that a MovieClip is in after it's created with createEmptyMovieClip.
 *     It is the only one not tested in this test. Instead, it gets tested in movieclip_default_state.
 * - Initial Loading State
 *     This state is entered when FP / Ruffle try to load the MovieClip. As soon as FP / Ruffle
 *     either load the first frame of the SWF or realise the movie can't be loaded, a different state
 *     is entered.
 *     Therefore, if FP / Ruffle are too fast to determine whether the file exists or not, the state
 *     can directly change after one frame from the default state to a different state.
 *     The initial loading state is different, depending on whether the SWF file which is loading is an
 *     online file or a local file.
 *     The local initial loading state is tested in this test.
 * - Error state
 *     This state is entered if no file could be loaded or if the loaded content is no valid supported
 *     content.
 *     It is tested in this test.
 * - Image state
 *     This state is entered if an image has been loaded.
 *     It is tested in this test.
 * - Success state
 *     This state is entered if the first frame of a valid SWF file has been loaded.
 *     It is tested in this test.
 * - Unloaded state
 *     This state is entered on the next frame after the movie has been unloaded.
 *     It is tested in this test.
 *
 * This test consists of several tests which each having a MovieClip go through these states and testing
 * if the values are correct.
 * More information about the details of each test is given as documentation of the function executing
 * the respective test.
 *
 * This test currently only tests the MovieClip states when trying to load a local file.
 * It still contains the test cases for remote URLs, however, they are not executed since Ruffle tests
 * currently don't support tests using remote URLs.
 * If Ruffle tests start supporting remote URLs (e.g. with a local server) at some point, these tests
 * can still be used.
 */

startNextTest(0);

/*
 * This function is called by each test and calls the next one.
 */
function startNextTest(currentTest:Number) {
	switch (currentTest) {
		case 0: testOfflineErrorCase(1); break;
		case 1: testOfflineErrorCase(2); break
		case 2: testOfflineSuccessCase(3); break;
		case 3: testOfflineSuccessCase(4); break
		case 4: trace("\nFinished all tests."); break;
		default: trace("\nError: Test number " + currentTest + " does not exist."); break;
	}
}



/*
 * Flash behaves the same if a file doesn't exist or isn't a valid file, so this tests both.
 * In this test (in FP), the MovieClip has four states:
 * The first (default) state is entered when creating an empty movie clip => _framesloaded == 1
 * The second (initial loading) state is entered when FP tries to load the MovieClip => _framesloaded == -1
 * The third (error) state is entered when FP realises the movie can't be loaded => _framesloaded == -1
 * The fourth (unloaded) state is entered on the next frame after the movie has been unloaded.
 * Ruffle is too fast to determine whether the file exists or not to (reliably) still be in the initial loading
 * state after one frame (Flash player often is too fast as well).
 */
function testOfflineErrorCase(currentTest:Number) {
	var mc: MovieClip;
	var mcPropsArray1: Array;
	var mcPropsArray2: Array;
	var mcPropsArray3: Array;
	var mcPropsArray4: Array;
	var mcPropsArray5: Array;
	var mcPropsArray6: Array;

	switch (currentTest) {
		case 1:
			trace("Test 1 (Offline; Link doesn't exist)");
			mc = createEmptyMovieClip("testMovieClip1", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie("no existing file.swf", mc);
			break;
		case 2:
			trace("\nTest 2 (Offline; Link is a text file)");
			mc = createEmptyMovieClip("testMovieClip2", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie("no correct file (text).swf", mc);
			break;
		default:
			trace("Error: Test number " + currentTest + " is invalid in this context.");
			return;
	}

	trace("loadMovie command issued.");
	mcPropsArray2 = getMcPropsArray(mc);
	compareMcProps(mcPropsArray1, mcPropsArray2, true);

	var frameCount = 0;
	var initialOneFinished = false;
	var errorCount = 0;
	var unloaded = false;
	this.onEnterFrame = function() {
		frameCount++;

		if (!unloaded) {
			if (mc._framesloaded == 1) {
				// Sometimes in Flash, the MovieClip is still in the first state for one frame.
				if (!initialOneFinished) {
					var changes = compareMcProps(mcPropsArray2, getMcPropsArray(mc), false);
					if (changes.length != 0) {
						trace("Error: _framesloaded must be -1.")
					}
					initialOneFinished = true;
				} else {
					trace("Error: _framesloaded must be -1.");
				}
			} else if (mc._framesloaded == -1) {
				// Flash uses _framesloaded == -1 both in the initial loading and in the error state for
				// offline files (which are identical).
				// Therefore, we wait 10 frames to be sure we're in the error state.
				errorCount++;
				if (errorCount == 1) {
					// This might be the initial loading state or (if Ruffle was too fast) the error state.
					trace("Frames loaded: -1 (1st time)");
					mcPropsArray3 = getMcPropsArray(mc);
					compareMcProps(mcPropsArray2, mcPropsArray3, true);
				} else if (errorCount >= 10) {
					// This is the error state. No matter which state was previously entered, nothing should
					// have changed.
					trace ("Frames loaded: -1 (10th time)")
					mcPropsArray4 = getMcPropsArray(mc);
					compareMcProps(mcPropsArray3, mcPropsArray4, true);

					// The MovieClip is unloaded and will enter the unloaded state on the next frame.
					unloadMovie(mc);
					unloaded = true;
					trace("unloadMovie command issued.")
					mcPropsArray5 = getMcPropsArray(mc);
					compareMcProps(mcPropsArray4, mcPropsArray5, true);
				}
			} else {
				trace("Error: _framesloaded must be -1.");
			}
		} else {
			mcPropsArray6 = getMcPropsArray(mc);
			compareMcProps(mcPropsArray5, mcPropsArray6, true);

			this.onEnterFrame = null;
			startNextTest(currentTest);
			return;
		}

		if (frameCount >= 15) {
			trace("Error: No error has occurred.");
			this.onEnterFrame = null;
			startNextTest(currentTest);
		}
	};
}


/*
 * Flash behaves mostly the same if a file is an image or a valid SWF file, so this tests both.
 * In this test (in FP), the MovieClip has four states:
 * The first (default) state is entered when creating an empty movie clip => _framesloaded == 1
 * The second (initial loading) state is entered when FP tries to load the MovieClip => _framesloaded == -1
 * The third (success / image) state is entered after FP loaded the first frame => _framesloaded == 1
 * (With real SWF files, _framesloaded doesn't go to one but jumps from 0 to a number and rises until
 * the full file is loaded, but an image only counts as one frame and the target.swf file only has one frame.)
 * The fourth (unloaded) state is entered on the next frame after the movie has been unloaded.
 * Ruffle is too fast to determine whether the file exists or not to (reliably) still be in the initial loading
 * state after one frame (Flash player often is too fast as well).
 */
function testOfflineSuccessCase(currentTest:Number) {
	var mc: MovieClip;
	var mcPropsArray1: Array;
	var mcPropsArray2: Array;
	var mcPropsArray3: Array;
	var mcPropsArray4: Array;
	var mcPropsArray5: Array;

	switch (currentTest) {
		case 3:
			trace("\nTest 3 (Offline; Link is an image)");
			mc = createEmptyMovieClip("testMovieClip3", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie("no correct file (image).swf", mc);
			break;
		case 4:
			trace("\nTest 4 (Offline; Link is a valid file)");
			mc = createEmptyMovieClip("testMovieClip4", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie("target.swf", mc);
			break;
		default:
			trace("Error: Test number " + currentTest + " is invalid in this context.");
			return;
	}

	trace("loadMovie command issued.");
	mcPropsArray2 = getMcPropsArray(mc);
	compareMcProps(mcPropsArray1, mcPropsArray2, true);

	var frameCount = 0;
	var initialOneFinished = false;
	var loadingStateStarted = false;
	var unloaded = false;
	this.onEnterFrame = function() {
		frameCount++;

		if (!unloaded) {
			if (mc._framesloaded == 1 && !initialOneFinished) {
				// Sometimes in Flash, the MovieClip is still in the first state for one frame.
				var changes = compareMcProps(mcPropsArray2, getMcPropsArray(mc), false);
				if (changes.length != 0) {
					// It's not the first state => We test if it's the success state.
					initialOneFinished = true;
				}
			} else if (mc._framesloaded == -1) {
				// Ruffle is (almost always) too fast to determine whether the file exists or not (and Flash often
				// is as well); therefore this won't trace anything if the state is correct.
				if (!loadingStateStarted) {
					var mcPropsArrayLoadingState = getMcPropsArray(mc);
					var differenceArray = compareMcProps(mcPropsArray2, mcPropsArrayLoadingState, false);
					var correctDifferenceArray = [["_framesloaded", 1, -1], ["_totalframes", 1, 0],
						["_url", "movieclip_state_values/test.swf", "movieclip_state_values/target.swf"],
						["getBytesTotal()", 0, -1], ["getSWFVersion()", 10, -1]];
					compareArrays(differenceArray, correctDifferenceArray, 2);
					loadingStateStarted = true;
				}
			} else if (mc._framesloaded != 1) {
				trace("Error: _framesloaded must be -1 or 1.");
			}
			if (mc._framesloaded == 1 && initialOneFinished) {
				trace("Frames loaded: 1");
				mcPropsArray3 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray2, mcPropsArray3, true);

				// The MovieClip is unloaded and will enter the unloaded state on the next frame.
				unloadMovie(mc);
				unloaded = true;
				trace("unloadMovie command issued.")
				mcPropsArray4 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray3, mcPropsArray4, true);
			}
			initialOneFinished = true;
		} else {
			mcPropsArray5 = getMcPropsArray(mc);
			compareMcProps(mcPropsArray4, mcPropsArray5, true);

			this.onEnterFrame = null;
			startNextTest(currentTest);
			return;
		}

		if (frameCount >= 10) {
			trace("Error: The MovieClip has not loaded.");
			this.onEnterFrame = null;
			startNextTest(currentTest);
		}
	};
}



/*
 * The following code is unused since Ruffle tests currently don't support tests using remote URLs.
 * It is kept here because Flash behaves differently with online SWFs and offline SWFs.
 * If Ruffle tests start supporting remote URLs (e.g. with a local server) at some point, these tests
 * can still be used.
 */

// TODO: Replace these with the valid domain / URL to the folder with the files.
var base_domain = "";
var base_url = "";

/*
 * Flash behaves the same if a file doesn't exist or isn't a valid file, so this tests both.
 * In this test (In FP), the MovieClip has four states:
 * The first (default) state is entered when creating an empty movie clip => _framesloaded == 1
 * The second (initial loading) state is entered when FP tries to load the MovieClip => _framesloaded == 0
 * The third (error) state is entered when FP realises the movie can't be loaded => _framesloaded == -1
 * The fourth (unloaded) state is entered on the next frame after the movie has been unloaded.
 * Flash often is too fast to determine whether the file exists or not to still be in the initial loading
 * state after one frame (Ruffle sometimes is too fast as well).
 */
function testOnlineErrorCase(currentTest:Number) {
	var mc: MovieClip
	var mcPropsArray1: Array;
	var mcPropsArray2: Array;
	var mcPropsArray3: Array;
	var mcPropsArray4: Array;
	var mcPropsArray5: Array;

	System.security.allowDomain(base_domain);

	switch (currentTest) {
		case 5:
			trace("Test 5 (Online; Link doesn't exist)");
			mc = createEmptyMovieClip("testMovieClip5", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie(base_url + "/no existing file.swf", mc);
			break;
		case 6:
			trace("\nTest 6 (Online; Link is a text file)");
			mc = createEmptyMovieClip("testMovieClip6", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie(base_url + "/no correct file (text).swf", mc);
			break;
		default:
			trace("Error: Test number " + currentTest + " invalid in this context.");
			return;
	}

	trace("loadMovie command issued.");
	mcPropsArray2 = getMcPropsArray(mc);
	compareMcProps(mcPropsArray1, mcPropsArray2, true);

	var frameCount = 0;
	var initialOneFinished = false;
	var loadingStateStarted = false;
	var unloaded = false;
	this.onEnterFrame = function() {
		frameCount++;

		if (!unloaded) {
			if (mc._framesloaded == 1) {
				// Sometimes in Flash, the MovieClip is still in the first state for one frame.
				if (!initialOneFinished) {
					var changes = compareMcProps(mcPropsArray2, getMcPropsArray(mc), false);
					if (changes.length != 0) {
						trace("Error: _framesloaded must be between -1 and 0.");
					}
					initialOneFinished = true;
				} else {
					trace("Error: _framesloaded must be between -1 and 0.");
				}
			} else if (mc._framesloaded == 0) {
				// Flash often is too fast to determine whether the file exists or not (and Ruffle sometimes
				// is as well); therefore this won't trace anything if the state is correct.
				if (!loadingStateStarted) {
					var mcPropsArrayLoadingState = getMcPropsArray(mc);
					var differenceArray = compareMcProps(mcPropsArray2, mcPropsArrayLoadingState, false);
					var correctDifferenceArray = [["_framesloaded", 1, 0], ["_totalframes", 1, 0]];
					compareArrays(differenceArray, correctDifferenceArray, 2);
					loadingStateStarted = true;
				}
			} else if (mc._framesloaded == -1) {
				trace("Frames loaded: -1");
				mcPropsArray3 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray2, mcPropsArray3, true);

				// The MovieClip is unloaded and will enter the unloaded state on the next frame.
				unloadMovie(mc);
				unloaded = true;
				trace("unloadMovie command issued.")
				mcPropsArray5 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray3, mcPropsArray4, true);
			} else {
				trace("Error: _framesloaded must be between -1 and 0.");
			}
		} else {
			mcPropsArray5 = getMcPropsArray(mc);
			compareMcProps(mcPropsArray4, mcPropsArray5, true);

			this.onEnterFrame = null;
			startNextTest(currentTest);
			return;
		}

		if (frameCount >= 10) {
			trace("Error: No error has occurred.");
			this.onEnterFrame = null;
			startNextTest(currentTest);
		}
	};
}


/*
 * Flash behaves mostly the same if a file is an image or a valid SWF file, so this tests both.
 * In this test (in FP), the MovieClip has three states:
 * The first (default) state is entered when creating an empty movie clip => _framesloaded == 1
 * The second (initial loading) state is entered when FP tries to load the MovieClip => _framesloaded == 0
 * The third (success) state is entered after FP loaded the first frame => _framesloaded == 1
 * (With real SWF files, _framesloaded doesn't go to one but jumps from 0 to a number and rises until
 * the full file is loaded, but an image only counts as one frame and the target.swf file only has one frame.)
 * Flash often is too fast to determine whether the file exists or not to still be in the initial loading
 * state after one frame (Ruffle sometimes is too fast as well).
 */
function testOnlineSuccessCase(currentTest:Number) {
	var mc: MovieClip;
	var mcPropsArray1: Array;
	var mcPropsArray2: Array;
	var mcPropsArray3: Array;
	var mcPropsArray4: Array;
	var mcPropsArray5: Array;

	System.security.allowDomain(base_domain);

	switch (currentTest) {
		case 7:
			trace("\nTest 7 (Online; Link is an image)");
			mc = createEmptyMovieClip("testMovieClip7", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie(base_url + "/no correct file (image).swf", mc);
			break;
		case 8:
			trace("\nTest 8 (Online; Link is a valid file)");
			mc = createEmptyMovieClip("testMovieClip8", getNextHighestDepth());
			mcPropsArray1 = getMcPropsArray(mc);

			loadMovie(base_url + "/target.swf", mc);
			break;
		default:
			trace("Error: Test number " + currentTest + " invalid in this context.");
			return;
	}

	trace("loadMovie command issued.");
	mcPropsArray2 = getMcPropsArray(mc);
	compareMcProps(mcPropsArray1, mcPropsArray2, true);

	var frameCount = 0;
	var initialOneFinished = false;
	var loadingStateStarted = false;
	var unloaded = false;
	this.onEnterFrame = function() {
		frameCount++;

		if (!unloaded) {
			if (mc._framesloaded == 1 && !initialOneFinished) {
				// Sometimes in Flash, the MovieClip is still in the first state for one frame.
				var changes = compareMcProps(mcPropsArray2, getMcPropsArray(mc), false);
				if (changes.length != 0) {
					// It's not the first state => We test if it's the success state.
					initialOneFinished = true;
				}
			} else if (mc._framesloaded == 0) {
				// Flash often is too fast to determine whether the file exists or not (and Ruffle sometimes
				// is as well); therefore this won't trace anything if the state is correct.
				if (!loadingStateStarted) {
					var mcPropsArrayLoadingState = getMcPropsArray(mc);
					var differenceArray = compareMcProps(mcPropsArray2, mcPropsArrayLoadingState, false);
					var correctDifferenceArray = [["_framesloaded", 1, 0], ["_totalframes", 1, 0]];
					compareArrays(differenceArray, correctDifferenceArray, 2);
					loadingStateStarted = true;
				}
			} else if (mc._framesloaded != 1) {
				trace("Error: _framesloaded must be between 0 and 1.");
			}
			if (mc._framesloaded == 1 && initialOneFinished) {
				trace("Frames loaded: 1");
				mcPropsArray3 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray2, mcPropsArray3, true);

				// The MovieClip is unloaded and will enter the unloaded state on the next frame.
				unloadMovie(mc);
				unloaded = true;
				trace("unloadMovie command issued.")
				mcPropsArray4 = getMcPropsArray(mc);
				compareMcProps(mcPropsArray3, mcPropsArray4, true);
			}
			initialOneFinished = true;
		} else {
			mcPropsArray5 = getMcPropsArray(mc);
			compareMcProps(mcPropsArray4, mcPropsArray5, true);

			this.onEnterFrame = null;
			startNextTest(currentTest);
			return;
		}

		if (frameCount >= 10) {
			trace("Error: The MovieClip has not loaded.");
			this.onEnterFrame = null;
			startNextTest(currentTest);
		}
	};
}



/*
 * This compares different arrays with a fixed dimension and traces an error if they are different.
 */
function compareArrays(array1:Array, array2:Array, arrayDimension:Number) {
	if (array1.length != array2.length) {
		trace("Error: The array lengths are not equal.")
		return;
	}

	for (var elementIterator = 0; elementIterator < array1.length; elementIterator++) {
		if (arrayDimension == 1) {
			if (array1[elementIterator] != array2[elementIterator]) {
				trace("Error: The arrays are not equal.");
				return;
			}
		} else {
			compareArrays(array1[elementIterator], array2[elementIterator], arrayDimension - 1);
		}
	}
}


/*
 * This compares two McProps-Arrays returned by getMcPropsArray.
 * If traceDifferences is true, all differences will be traced.
 * It returns the diverging values with their respective names in a two-dimensional array.
 */
function compareMcProps(mcPropsArray1:Array, mcPropsArray2:Array, traceDifferences:Boolean) {
	if (mcPropsArray1 == undefined || mcPropsArray2 == undefined) {
		trace("Error: An mcPropsArray is undefined.");
		return;
	}

	var differenceArray = [];
	for (var propIterator = 0; propIterator < mcPropsArray1.length; propIterator++) {
		if (mcPropsArray1[propIterator][1].toString() != mcPropsArray2[propIterator][1].toString()) {
			var value1 = mcPropsArray1[propIterator][1];
			var value2 = mcPropsArray2[propIterator][1];
			var propName = mcPropsArray1[propIterator][0];
			differenceArray.push([propName, value1, value2])
			if (traceDifferences) {
				trace("Change: Prop " + propName + " is \"" + value1 + "\" on the first, but \"" + value2 +
					"\" on the second target.");
			}
		}
	}
	if (traceDifferences && differenceArray.length == 0) {
		trace("Both targets have the same props.");
	}

	return differenceArray;
}


/*
 * This returns all properties and the results of some getter functions of a MovieClip with their respective
 * names in a two-dimensional array.
 */
function getMcPropsArray(mc:MovieClip) {
	var mcProps = [];
	var simplePropNames = ["_accProps", "_alpha", "_currentframe", "_droptarget", "_focusrect", "_framesloaded",
		"_height", "_highquality", "_lockroot", "_name", "_parent", "_quality", "_rotation", "_soundbuftime",
		"_target", "_totalframes", "_visible", "_width", "_x", "_xmouse", "_xscale", "_y", "_ymouse", "_yscale",
		"blendMode", "cacheAsBitmap", "enabled", "filters", "filters.length", "focusEnabled", "forceSmoothing",
		"hitArea", "menu", "opaqueBackground", "scale9Grid", "scrollRect", "tabChildren", "tabEnabled", "tabIndex",
		"trackAsMenu", "transform.colorTransform", "transform.concatenatedColorTransform", "transform.matrix",
		"transform.concatenatedMatrix", "transform.pixelBounds", "useHandCursor"];
	for (var simplePropIterator = 0; simplePropIterator < simplePropNames.length; simplePropIterator++) {
		var simplePropName = simplePropNames[simplePropIterator];
		var simplePropValue = eval("mc." + simplePropName);
		mcProps.push([simplePropName, simplePropValue]);
	}

	var url = unescape(mc._url);
	if (url.indexOf("file:///") == 0) {
		var urlSplit = url.split("/");
		mcProps.push(["_url", urlSplit[urlSplit.length - 1]]);
	} else {
		mcProps.push(["_url", url]);
	}

	var getBoundsThis = mc.getBounds(this);
	mcProps.push(["getBounds(this).xMin", getBoundsThis.xMin]);
	mcProps.push(["getBounds(this).xMax", getBoundsThis.xMax]);
	mcProps.push(["getBounds(this).yMin", getBoundsThis.yMin]);
	mcProps.push(["getBounds(this).yMax", getBoundsThis.yMax]);
	var getBoundsMc = mc.getBounds(mc);
	mcProps.push(["getBounds(mc).xMin", getBoundsMc.xMin]);
	mcProps.push(["getBounds(mc).xMax", getBoundsMc.xMax]);
	mcProps.push(["getBounds(mc).yMin", getBoundsMc.yMin]);
	mcProps.push(["getBounds(mc).yMax", getBoundsMc.yMax]);
	mcProps.push(["getBytesLoaded()", mc.getBytesLoaded()]);
	mcProps.push(["getBytesTotal()", mc.getBytesTotal()]);
	mcProps.push(["getDepth()", mc.getDepth()]);
	mcProps.push(["getInstanceAtDepth(0)", mc.getInstanceAtDepth(0)]);
	mcProps.push(["getNextHighestDepth()", mc.getNextHighestDepth()]);
	var getRectThis = mc.getBounds(this);
	mcProps.push(["getRect(this).xMin", getRectThis.xMin]);
	mcProps.push(["getRect(this).xMax", getRectThis.xMax]);
	mcProps.push(["getRect(this).yMin", getRectThis.yMin]);
	mcProps.push(["getRect(this).yMax", getRectThis.yMax]);
	var getRectMc = mc.getBounds(mc);
	mcProps.push(["getRect(mc).xMin", getRectMc.xMin]);
	mcProps.push(["getRect(mc).xMax", getRectMc.xMax]);
	mcProps.push(["getRect(mc).yMin", getRectMc.yMin]);
	mcProps.push(["getRect(mc).yMax", getRectMc.yMax]);
	mcProps.push(["getSWFVersion()", mc.getSWFVersion()]);
	mcProps.push(["getTextSnapshot().getCount()", mc.getTextSnapshot().getCount()]);

	return mcProps;
}
