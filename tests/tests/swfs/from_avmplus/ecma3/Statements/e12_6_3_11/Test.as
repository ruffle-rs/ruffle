/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "12.6.3-11";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

//    5.  Get the name of the next property of Result(3) that doesn't have the
//        DontEnum attribute. If there is no such property, go to step 14.

    var result = "";

    for ( p in Number ) { result += String(p) };

    array[item++] = Assert.expectEq( 
        "result = \"\"; for ( p in Number ) { result += String(p) };",
        "",
        result );
    return array;
}
