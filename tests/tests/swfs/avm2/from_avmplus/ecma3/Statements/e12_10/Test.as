/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "12.10-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The with statement";

    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

    var x;
    with(7) x = valueOf();
    array[item++] = Assert.expectEq(   
                                    "var x; with (7) x = valueOf(); typeof x;",
                                    "number",
                                    (typeof x) );
    //print( "FAILED: bug 103243 filed" );

    return ( array );
}

