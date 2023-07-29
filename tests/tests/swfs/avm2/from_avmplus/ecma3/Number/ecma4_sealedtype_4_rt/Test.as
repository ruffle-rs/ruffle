/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
// var SECTION = "ecma4_selaedtype_4_rt";

var testcases = getTestCases();

              // displays results.
              

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var n = 5;
    var expected = "error";
    var exception;

    try {
        n.someVar = 3;
        actual = "no error";
    } catch(e1) {
        actual = "error";
        exception = e1.toString();
    }

    array[item++] = Assert.expectEq( "n = 5, n.someVar = 3: ", "ReferenceError: Error #1056", Utils.referenceError(exception));

    try {
        v = n.someVar;
        actual = "no error";
    } catch(e2) {
        actual = "error";
        exception = e2.toString();
    }

    array[item++] = Assert.expectEq( "n = 5, v = n.someVar: ", "ReferenceError: Error #1069", Utils.referenceError(exception));

    return array;
}
