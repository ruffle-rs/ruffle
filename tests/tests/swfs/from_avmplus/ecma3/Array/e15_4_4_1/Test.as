/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.4.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype.constructor";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    /* CN:  Not true for ES4, Array.prototype is a vanilla Object, its constructor is Object. */
    array[item++] = Assert.expectEq(   "Array.prototype.constructor == Array", true,   Array.prototype.constructor == Array);
    //array[item++] = Assert.expectEq(     "Array.prototype.constructor == Object", true,   Array.prototype.constructor == Object);

    return ( array );
}
