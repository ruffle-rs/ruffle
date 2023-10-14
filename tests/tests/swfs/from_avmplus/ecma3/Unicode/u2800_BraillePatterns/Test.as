/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

include "../include/unicodeUtil.as";
include "../include/unicodeNegativeUtil.as";

// var SECTION = "Braille Patterns";
// var VERSION = "ECMA_3";
// var TITLE = "Test String functions (search, match, split, replace) on all unicode characters";


var array = new Array();
var item = 0;
getTestCases();

var testcases = array;

function getTestCases():void {
  // Braille Patterns
  testUnicodeRange(0x2800, 0x28FF);
  negativeTestUnicodeRange(0x2800, 0x28FF);
}
