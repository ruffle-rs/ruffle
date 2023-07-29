/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "while-001";
//     var VERSION = "ECMA_2";
//     var TITLE   = "while statement";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    DoWhile();

    function DoWhile() {
        result = "pass";

        while (false) {
            result = "fail";
            break;
        }

        array[item++] = Assert.expectEq(
            
            "while statement: don't evaluate statement is expression is false",
            "pass",
            result );

    }
    return array;
}
