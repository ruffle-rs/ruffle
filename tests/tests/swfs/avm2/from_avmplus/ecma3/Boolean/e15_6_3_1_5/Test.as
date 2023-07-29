/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var VERSION = "ECMA_2";
// var SECTION = "15.6.3.1-5";
// var TITLE   = "Boolean.prototype"

var tc= 0;
var testcases = getTestCases();

// All tests must call a function that returns an array of TestCase objects.

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "Class.constructor.prototype == Boolean.constructor.prototype", true, Class.constructor.prototype == Boolean.constructor.prototype );

    return ( array );

}
