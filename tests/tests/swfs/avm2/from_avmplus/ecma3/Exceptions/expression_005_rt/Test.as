/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "expression-005";
//     var VERSION = "JS1_4";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var result = "Failed";
    var expect = "Passed";
    var exception = "No exception thrown";

    try {
        result = new Math();
    } catch ( e:TypeError ) {
        result = expect;
        exception = e.toString();
    }finally{

    array[item++] = Assert.expectEq(
     // //    SECTION,
        "result= new Math() (threw " + Utils.typeError(exception) + ": Math is not a constructor)",
        expect,
        result );
             }

    return array;
}
