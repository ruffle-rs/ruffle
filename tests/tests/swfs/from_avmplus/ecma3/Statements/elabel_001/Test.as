/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "label-003";
//     var VERSION = "ECMA_2";
//     var TITLE   = "Labeled statements";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    LabelTest(0, 0);
    LabelTest(1, 1)
    LabelTest(-1, 1000);
    LabelTest(false,  0);
    LabelTest(true, 1);


    function LabelTest( limit, expect) {
        woo: for ( var result = 0; result < 1000; result++ ) { if (result == limit) { break woo; } else { continue woo; } };

        array[item++] = Assert.expectEq(
            
            "break out of a labeled for loop: "+ limit,
            expect,
            result );
    }
    return array;
}
