/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "expression-016";
//     var VERSION = "JS1_4";
//     var TITLE   = "Function Calls";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";

    try {
        result = (void 0).valueOf();
    } catch ( e:TypeError ) {
        result = expect;
        exception = e.toString();
    }

    array[item++] = Assert.expectEq(
     // //    SECTION,
        "(void 0).valueOf()" +
        " (threw " + Utils.typeError(exception) +"Attempted to create an object of a variable that is not a function/class)",
        expect,
        result );

    return array;
}
