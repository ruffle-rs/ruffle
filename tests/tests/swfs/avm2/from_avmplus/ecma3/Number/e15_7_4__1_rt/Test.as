/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.7.4-1";
//     var VERSION = "ECMA_1";

    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq( "Number.prototype.valueOf()",      0,                  Number.prototype.valueOf() );
    array[item++] = Assert.expectEq( "typeof(Number.prototype)",        "object",           typeof(Number.prototype) );
    array[item++] = Assert.expectEq( "Number.prototype.constructor == Number",    true,     Number.prototype.constructor == Number );
//    array[item++] = Assert.expectEq( "Number.prototype == Number.__proto__",      true,   Number.prototype == Number.__proto__ );
    return ( array );
}
