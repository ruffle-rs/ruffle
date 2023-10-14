/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.7.4.3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();



function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;

    var o:* = new Object();
    o.toString = Number.prototype.toString;
    array[item++] = Assert.expectEq(  "o = new Object(); o.toString = Number.prototype.toString; o.toLocaleString()",  "[object Object]",    o.toLocaleString() );
//    array[item++] = Assert.expectEq(  "o = new String(); o.toString = Number.prototype.toString; o.toLocaleString()",  "error",    "o = new String(); o.toString = Number.prototype.toString; o.toLocaleString()" );
//    array[item++] = Assert.expectEq(  "o = 3; o.toString = Number.prototype.toString; o.toLocaleString()",             "error",    "o = 3; o.toString = Number.prototype.toString; o.toLocaleString()" );

    return ( array );
}
