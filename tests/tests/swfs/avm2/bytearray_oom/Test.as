package {import flash.display.MovieClip;public class Test extends MovieClip {}}

import flash.utils.ByteArray;

try {
    new ByteArray().length = 0xFFFFFFFF;
    trace("Success");
} catch (e) {
    trace(e.getStackTrace());
}
