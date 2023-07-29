/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.6.4.3-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Boolean.prototype.valueOf()";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "new Boolean(1)",       true,   (new Boolean(1)).valueOf() );

    array[item++] = Assert.expectEq(    "new Boolean(0)",       false,  (new Boolean(0)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(-1)",      true,   (new Boolean(-1)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean('1')",     true,   (new Boolean("1")).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean('0')",     true,   (new Boolean("0")).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(true)",    true,   (new Boolean(true)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(false)",   false,  (new Boolean(false)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean('true')",  true,   (new Boolean("true")).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean('false')", true,   (new Boolean('false')).valueOf() );

    array[item++] = Assert.expectEq(    "new Boolean('')",      false,  (new Boolean('')).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(null)",    false,  (new Boolean(null)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(void(0))", false,  (new Boolean(void(0))).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(-Infinity)", true, (new Boolean(Number.NEGATIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(NaN)",     false,  (new Boolean(Number.NaN)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean()",        false,  (new Boolean()).valueOf() );

    array[item++] = Assert.expectEq(    "new Boolean(x=1)",     true,   (new Boolean(x=1)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x=0)",     false,  (new Boolean(x=0)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x=false)", false,  (new Boolean(x=false)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x=true)",  true,   (new Boolean(x=true)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x=null)",  false,  (new Boolean(x=null)).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x='')",    false,  (new Boolean(x="")).valueOf() );
    array[item++] = Assert.expectEq(    "new Boolean(x=' ')",   true,   (new Boolean(x=" ")).valueOf() );

    return ( array );
}
