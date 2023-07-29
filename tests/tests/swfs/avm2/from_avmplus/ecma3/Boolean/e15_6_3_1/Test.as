/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.6.3.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Boolean.prototype";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "Boolean.prototype.valueOf()",false,   Boolean.prototype.valueOf());
    array[item++] = Assert.expectEq(   "Boolean.length",          1,       Boolean.length );

    return ( array );
}
