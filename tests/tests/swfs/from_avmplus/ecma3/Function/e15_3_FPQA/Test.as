/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "test";       // provide a document reference (ie, ECMA section)
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "test";       // Provide ECMA section title or a description
var BUGNUMBER = "";

var testcases = getTestCases();
              // displays results.

// ******************************************************
// adds test case for following condition not included
// in the mozilla.org tests
//
// (function() { print( "hello world");})();
// ******************************************************

function getTestCases() {
    var array = new Array();
    var item = 0;
    var MYVAR = "FAILED";
    array[item++] = Assert.expectEq( "function(){ MYVAR='PASSED';})()", "PASSED", ((function() { MYVAR = "PASSED";})(), MYVAR) );
    array[item++] = Assert.expectEq( "function(){ MYVAR='PASSED';})()", "PASSED", ((function(param1) { MYVAR = param1;})("PASSED"), MYVAR) );
    return array;
}
