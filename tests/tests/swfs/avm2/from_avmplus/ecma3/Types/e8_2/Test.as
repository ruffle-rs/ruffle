/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "8.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The null type";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var x = null;
    array[item++] = Assert.expectEq( 
                                    "var x = null; typeof x",
                                    null,
                                    x);

    array[item++] = Assert.expectEq( 
                                    "typeof null",
                                    "object",
                                    typeof null);
    return array;
}
