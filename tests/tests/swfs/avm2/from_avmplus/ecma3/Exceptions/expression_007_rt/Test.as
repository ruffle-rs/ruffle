/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "expression-007";
//     var VERSION = "JS1_4";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";
    var UNDEFINED = void 0;
    
    try {
        
          
        UNDEFINED();
    } catch ( e:TypeError ) {
        
        exception = e.toString();
    }finally{

    array[item++] = Assert.expectEq(
     // //    SECTION,
        "UNDEFINED = void 0; result = UNDEFINED()" +
        " (threw " + Utils.typeError(exception) +": Call attempted on an object that is not a function.)",
        "TypeError: Error #1006",
        Utils.typeError(exception) );

    }
    return array;
}
