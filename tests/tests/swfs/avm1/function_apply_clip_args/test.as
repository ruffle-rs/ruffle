// Tests Function.apply with a movie clip that has `length`, `0` and `1`
// properties as the arguments, compared with an array and a plain object.

function describeArguments() {
    return arguments.length + ": " + arguments.join(", ");
}

createEmptyMovieClip("clip", getNextHighestDepth());
clip.length = 2;
clip[0] = "clip0";
clip[1] = "clip1";

var object = {length: 2};
object[0] = "object0";
object[1] = "object1";

trace("apply(null, clip): " + describeArguments.apply(null, clip));
trace("apply(null, object): " + describeArguments.apply(null, object));
trace("apply(null, array): " + describeArguments.apply(null, ["array0", "array1"]));
