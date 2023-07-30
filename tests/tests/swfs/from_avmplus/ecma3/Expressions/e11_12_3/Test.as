/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "11_12_3";
//     var VERSION = "ECMA_1";

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    // the following expression should NOT be an error in JS.
    var MYVAR =  true ? ('FAIL1', 'PASSED') : 'FAIL2';
    array[item++] = Assert.expectEq( 
                                    "var MYVAR =  true ? ('FAIL1', 'PASSED') : 'FAIL2'; MYVAR",
                                    "PASSED",
                                    MYVAR );

    // get around potential parse time error by putting expression in an eval statement

    //array[tc].actual = array[tc].actual;
    return (array);
}
