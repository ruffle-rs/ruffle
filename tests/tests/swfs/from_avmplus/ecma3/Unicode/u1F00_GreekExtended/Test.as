/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

include "../include/unicodeUtil.as";
include "../include/unicodeNegativeUtil.as";

// var SECTION = "Greek Extended";
// var VERSION = "ECMA_3";
// var TITLE = "Test String functions (search, match, split, replace) on all unicode characters";


var array = new Array();
var item = 0;
getTestCases();

var testcases = array;

function getTestCases():void {
  // Greek Extended
  testUnicodeRange(0x1F00, 0x1FFF);
  negativeTestUnicodeRange(0x1F00, 0x1FFF);
}
