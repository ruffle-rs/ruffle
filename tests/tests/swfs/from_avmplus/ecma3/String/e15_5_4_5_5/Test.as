/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.5-5";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charCodeAt";


    var TEST_STRING = "";

    for ( var i = 0x0000; i < 255; i++ ) {
        TEST_STRING += String.fromCharCode( i );
    }



    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var origBooleanCharCodeAt = Boolean.prototype.charCodeAt;
    Boolean.prototype.charCodeAt=String.prototype.charCodeAt;

    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(0)", 0x0074,    (x = new Boolean(true), x.charCodeAt(0)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(1)", 0x0072,    (x = new Boolean(true), x.charCodeAt(1)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(2)", 0x0075,    (x = new Boolean(true), x.charCodeAt(2)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(3)", 0x0065,    (x = new Boolean(true), x.charCodeAt(3)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(4)", Number.NaN,     (x = new Boolean(true), x.charCodeAt(4)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(-1)", Number.NaN,    (x = new Boolean(true), x.charCodeAt(-1)) );

    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(true)",  0x0072,    (x = new Boolean(true), x.charCodeAt(true)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(false)", 0x0074,    (x = new Boolean(true), x.charCodeAt(false)) );

    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(0)",    Number.NaN,     (x=new String(),x.charCodeAt(0)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(1)",    Number.NaN,     (x=new String(),x.charCodeAt(1)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(-1)",   Number.NaN,     (x=new String(),x.charCodeAt(-1)) );

    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(NaN)",  Number.NaN,     (x=new String(),x.charCodeAt(Number.NaN)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(Number.POSITIVE_INFINITY)",   Number.NaN,     (x=new String(),x.charCodeAt(Number.POSITIVE_INFINITY)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(Number.NEGATIVE_INFINITY)",   Number.NaN,     (x=new String(),x.charCodeAt(Number.NEGATIVE_INFINITY)) );

    for ( var j = 0; j < 255; j++ ) {
        array[item++] = Assert.expectEq(   "TEST_STRING.charCodeAt("+j+")",    j,     TEST_STRING.charCodeAt(j) );
    }
    
    Boolean.prototype.charCodeAt = origBooleanCharCodeAt;
    
    return (array );
}
