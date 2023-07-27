/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "8.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The undefined type";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var x;
    array[item++] = Assert.expectEq( 
                                    "var x; typeof x",
                                    "undefined",
                                     typeof x);
    var x;
    array[item++] = Assert.expectEq( 
                                    "var x; typeof x == 'undefined",
                                    true,
                                    typeof x == 'undefined');
    var x;
    array[item++] = Assert.expectEq( 
                                    "var x; x == void 0",
                                    true,
                                    x == void 0);
    return array;
}
