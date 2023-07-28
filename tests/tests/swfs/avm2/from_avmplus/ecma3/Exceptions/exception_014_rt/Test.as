/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "exception-014";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Regression test for bug 105083";
    var BUGNUMBER= "105083";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    try {
        throw "Exception";
    } catch ( e ) {
        array[item++] = Assert.expectEq( "first thrown Exception", "Exception", e.toString() );
    }

    try {
        throw "Exception";
    } catch ( e1 ) {
        array[item++] = Assert.expectEq( "first thrown Exception", "Exception", e1.toString() );
    }
    return array;
}
