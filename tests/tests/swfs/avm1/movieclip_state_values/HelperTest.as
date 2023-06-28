/*
 * This test is a HelperTest which can be used to find out about how the state of a MovieClip changes
 * when trying to load and unload it.
 *
 * The test creates a MovieClip and tries to load it. It traces all state variables that change each frame.
 * On the 20th frame, it unloads the MovieClip. On the 35th frame, it finishes.
 *
 * This is what the different MovieClip states have originally been identified with.
 * It is kept because it might still be helpful in the future, e.g. to detect differences in the MovieClip
 * states depending on the SWF versions and the Flash Player version.
 */

test()

/*
 * This is the actual test.
 */
function test() {
	var mcPropsArray1:Array;
	var mcPropsArray2:Array;
	var mcPropsArray3:Array;

	var mc = createEmptyMovieClip("clip", getNextHighestDepth());
	mcPropsArray1 = getMcPropsArray(mc);

	loadMovie("no existing file.swf", mc); // Change this line
	trace("loadMovie command issued.");
	mcPropsArray2 = getMcPropsArray(mc);
	compareMcProps(mcPropsArray1, mcPropsArray2, true);


	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		trace("Frame: " + frameCount);
		mcPropsArray3 = getMcPropsArray(mc);
		compareMcProps(mcPropsArray2, mcPropsArray3, true);
		mcPropsArray2 = mcPropsArray3;

		if (frameCount == 20) {
			unloadMovie(mc);
			trace("Unloaded");
			mcPropsArray3 = getMcPropsArray(mc);
			compareMcProps(mcPropsArray2, mcPropsArray3, true);
			mcPropsArray2 = mcPropsArray3;
			trace("Unloaded 2");
		}
		if (frameCount >= 35) {
			trace("Finished.");
			this.onEnterFrame = null;
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
		mcProps.push(["_url", urlSplit[urlSplit.length - 2] + "/" + urlSplit[urlSplit.length - 1]]);
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
