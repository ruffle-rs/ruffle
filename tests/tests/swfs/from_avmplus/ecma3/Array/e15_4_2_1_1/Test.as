/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.2.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor:  new Array( item0, item1, ...)";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    var arr;

    array[item++] = Assert.expectEq(   "typeof new Array(1,2)",        "object",           typeof new Array(1,2) );
    array[item++] = Assert.expectEq(   "(new Array(1,2)).toString",    "function Function() {}",    ((new Array(1,2)).toString).toString() );
    array[item++] = Assert.expectEq( 
                                    "var arr = new Array(1,2,3); arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (arr = new Array(1,2,3), arr.getClass = Object.prototype.toString, arr.getClass() ) );

    array[item++] = Assert.expectEq(   "(new Array(1,2)).length",      2,                  (new Array(1,2)).length );
    array[item++] = Assert.expectEq(   "var arr = (new Array(1,2)), arr[0]",  1,           (arr = (new Array(1,2)), arr[0] ) );
    array[item++] = Assert.expectEq(   "var arr = (new Array(1,2)), arr[1]",  2,           (arr = (new Array(1,2)), arr[1] ) );
    array[item++] = Assert.expectEq(   "var arr = (new Array(1,2)), String(arr)",  "1,2",  (arr = (new Array(1,2)), String(arr) ) );

    return ( array );
}
