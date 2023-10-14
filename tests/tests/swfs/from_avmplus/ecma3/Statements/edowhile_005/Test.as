/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "dowhile-005";
//     var VERSION = "ECMA_2";
//     var TITLE   = "do...while with a labeled continue statement";
    var BUGNUMBER = "316293";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    NestedLabel();


    function NestedLabel() {
        i = 0;
        result1 = "pass";
        result2 = "fail: did not hit code after inner loop";
        result3 = "pass";

        outer: {
            do {
                inner: {
//                    print( i );
                    break inner;
                    result1 = "fail: did not break out of inner label";
                  }
                result2 = "pass";
                break outer;
                trace(i);
            } while ( i++ < 100 );

        }

        result3 = "fail: did not break out of outer label";

        array[item++] = Assert.expectEq(
            
            "number of loop iterations",
            0,
            i );

        array[item++] = Assert.expectEq(
            
            "break out of inner loop",
            "pass",
            result1 );

        array[item++] = Assert.expectEq(
            
            "break out of outer loop",
            "pass",
            result2 );
    }
    return array;
}
