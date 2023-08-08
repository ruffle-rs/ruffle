// SWF Version 10

/*
 * This test tests the default state and the unloaded state of a child MovieClip that's loaded
 * from the library.
 * A state of a MovieClip consists of the values of all properties and the results of some getter
 * functions of the MovieClip.
 *
 * The default state is the state the MovieClip is in after it's loaded with attachMovie.
 * The unloaded state is the state the MovieClip enters on the next frame after the MovieClip
 * has been unloaded.
 */

var childClip = this.attachMovie("childClip", "childClip", this.getNextHighestDepth());

var mcPropsArray1: Array;
var mcPropsArray2: Array;
var mcPropsArray3: Array;
var frameCount = 0;
this.onEnterFrame = function() {
	frameCount++;

	if (frameCount == 1) {
		mcPropsArray1 = getMcPropsArray(childClip);
		printMcProps(mcPropsArray1);

		unloadMovie(childClip);

		mcPropsArray2 = getMcPropsArray(childClip);
		trace("");
		compareMcProps(mcPropsArray1, mcPropsArray2, true);
	}

	if (frameCount == 2) {
		mcPropsArray3 = getMcPropsArray(childClip);
		trace("");
		compareMcProps(mcPropsArray2, mcPropsArray3, true);
		this.onEnterFrame = null;
	}
};



/*
 * This traces all elements of an McProps-Array returned by getMcPropsArray with their respective names.
 */
function printMcProps(mcPropsArray:Array) {
	for (var propIterator = 0; propIterator < mcPropsArray.length; propIterator++) {
		var propName = mcPropsArray[propIterator][0];
		var propValue = mcPropsArray[propIterator][1];
		trace(propName + " = " + propValue);
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
