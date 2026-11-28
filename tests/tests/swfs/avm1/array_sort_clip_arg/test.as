// Tests Array.sort with a movie clip as the first argument, alone and with
// sort options, and traces the return value and the array.

createEmptyMovieClip("clip", getNextHighestDepth());

var optionsList = [0, Array.DESCENDING, Array.NUMERIC, Array.RETURNINDEXEDARRAY];

for (var i = 0; i < optionsList.length; i++) {
    var options = optionsList[i];
    var array = [3, 10, 1, 2];
    var result = array.sort(clip, options);
    trace("sort(clip, " + options + "): " + result + " / " + array);
}

var array = [3, 10, 1, 2];
var result = array.sort(clip);
trace("sort(clip): " + result + " / " + array);
