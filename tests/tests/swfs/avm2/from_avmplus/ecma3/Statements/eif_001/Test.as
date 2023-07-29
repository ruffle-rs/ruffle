/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "for-001";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The if  statement";
    var BUGNUMBER="148822";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var a = 0;
    var b = 0;
    var result = "passed";

    if ( a = b ) {
        result = "failed:  a = b should return 0";
    }

    array[item++] = Assert.expectEq(
        
        "if ( a = b ), where a and b are both equal to 0",
        "passed",
        result );
    return array;
}
