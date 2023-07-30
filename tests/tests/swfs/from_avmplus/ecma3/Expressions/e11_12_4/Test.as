/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "11_12_4";
//     var VERSION = "ECMA_1";

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    // the following expression should NOT be an error in JS.
    true ? MYVAR1 = 'PASSED' : MYVAR1 = 'FAILED'
    array[item++] = Assert.expectEq( 
                                    "true ? MYVAR1 = 'PASSED' : MYVAR1 = 'FAILED'; MYVAR1",
                                    "PASSED",
                                    MYVAR1);

    // get around potential parse time error by putting expression in an eval statement

    //array[tc].actual = ( array[tc].actual );
    return (array);
}
