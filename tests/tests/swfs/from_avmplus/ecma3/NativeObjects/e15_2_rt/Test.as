/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Native ECMAScript Objects";


    var testcases = getTestCases();
function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "Object.constructor.prototype",   Object.prototype+"",   Object.constructor.prototype+"" );
    array[item++] = Assert.expectEq(  "Array.constructor.prototype",    Object.prototype+"",   Array.constructor.prototype+"");
    array[item++] = Assert.expectEq(  "String.constructor.prototype",   Object.prototype+"",   String.constructor.prototype+"");
    array[item++] = Assert.expectEq(  "Boolean.constructor.prototype", Object.prototype+"",   Boolean.constructor.prototype+"");
    array[item++] = Assert.expectEq(   "Number.constructor.prototype", Object.prototype+"",   Number.constructor.prototype+"");
    array[item++] = Assert.expectEq(   "Date.constructor.prototype",   Object.prototype+"",   Date.constructor.prototype+"");
    array[item++] = Assert.expectEq(  "getTestCases.constructor.prototype", Function.prototype+"", getTestCases.constructor.prototype+"");
    array[item++] = Assert.expectEq(  "Math.pow.constructor.prototype", Function.prototype+"", Math.pow.constructor.prototype+"");
    array[item++] = Assert.expectEq(  "String.prototype.indexOf.constructor.prototype", Function.prototype+"",   String.prototype.indexOf.constructor.prototype+"");

    return ( array );
}
