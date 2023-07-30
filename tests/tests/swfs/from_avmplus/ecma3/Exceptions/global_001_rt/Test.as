/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 // TODO: REVIEW AS4 CONVERSION ISSUE 
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "global-001";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Global Object";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";

    try {
        result = new this();
    } catch ( e:TypeError ) {
        result = expect;
        exception = e.toString();
    }

    array[item++] = Assert.expectEq(
        // SECTION,
        "result = new this()" +
        " (threw " + Utils.typeError(exception) +")",
        expect,
        result );
        
    return array;
}
