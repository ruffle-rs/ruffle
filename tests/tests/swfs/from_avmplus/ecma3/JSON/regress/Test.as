/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "JSON";
// var VERSION = "AS3";
// var TITLE   = "JSON regression tests";


// Bug 640711, comment 58.  \u#### emission was clobbering both
// arbitrary memory due to an unitialized variable and forgetting
// to finish emitting the remainder of the string.

Assert.expectEq("JSON.stringify('mn\\u0001op\\u0002qr\\u0003st')",
            '"mn\\u0001op\\u0002qr\\u0003st"',
            JSON.stringify('mn\u0001op\u0002qr\u0003st'));

// Bugzilla 672484: The string literal "\u000FF " *is* valid JSON input.
Assert.expectEq("JSON.parse('\"\\u000FF \"')", "\u000FF ", JSON.parse('"\\u000FF "'));
Assert.expectEq("JSON.parse('\"\\u0061F \"')", "aF ", JSON.parse('"\\u0061F "'));

// Another problem: the JSON.parse and JSON.stringify routines
// are written as recursive procedures.  How do they handle
// inputs where the recursion must descend deep into the structure?

// This loop takes a second or less on my iMac.
// Also, on my iMac, a limit of 1e4 is too small to expose the bug.

var deep = {};
for (var i = 0; i < 1e3; i++) {
    var y = {};
    y.next = deep;
    deep = y;
}

function drop(x) { return "okay"; }
exception1 = 'no exception';
var result;
try {
    result = drop(JSON.stringify(deep));
} catch (e) {
    exception1 = Utils.grabError(e,e.toString());
    if (exception1 == "Error #1023")
        result = "okay";
}
Assert.expectEq("JSON.stringify(deep)", "okay", result);


var prefix = '{"next":';
var suffix = '}';
for (var i= 0; i < Math.log(1e3)/Math.log(2); i++) {
    prefix = prefix + prefix
    suffix = suffix + suffix;
}
var deep2 = prefix + '{}' + suffix;

exception2 = 'no exception';
try {
    result = JSON.parse(deep2);
    result = "okay";
} catch (e) {
    exception2 = Utils.grabError(e,e.toString());
    if (exception2 == "Error #1023")
    result = "okay";
}
Assert.expectEq("JSON.parse(deep2)", "okay", result);

