/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "expression-009";
//     var VERSION = "JS1_4";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var STRING = "";

    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";

    try {
        result = new STRING();
    } catch ( e:TypeError ) {
        result = expect;
        exception = e.toString();
    }

    array[item++] = Assert.expectEq(
     // //    SECTION,
        "STRING = ''; result = new STRING()" +
        " (threw " + Utils.typeError(exception) +": Attempted to create a new object of a variable)",
        expect,
        result );
    return array;
}
