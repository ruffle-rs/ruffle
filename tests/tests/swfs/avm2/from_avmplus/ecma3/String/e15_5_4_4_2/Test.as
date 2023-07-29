/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.4-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charAt";

    //writeHeaderToLog( SECTION + " "+ TITLE);

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

  /*array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(0)", "t",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(0)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(1)", "r",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(1)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(2)", "u",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(2)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(3)", "e",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(3)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(4)", "",     (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(4)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(-1)", "",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(-1)) );

    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(true)", "r",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(true)) );
    array[item++] = Assert.expectEq(      "x = new Boolean(true); x.charAt=String.prototype.charAt;x.charAt(false)", "t",    (x = new Boolean(true), x.charAt=String.prototype.charAt,x.charAt(false)) );*/

    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(0)",    "",     (x=new String(),x.charAt(0)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(1)",    "",     (x=new String(),x.charAt(1)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(-1)",   "",     (x=new String(),x.charAt(-1)) );

    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(NaN)",  "",     (x=new String(),x.charAt(Number.NaN)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(Number.POSITIVE_INFINITY)",   "",     (x=new String(),x.charAt(Number.POSITIVE_INFINITY)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charAt(Number.NEGATIVE_INFINITY)",   "",     (x=new String(),x.charAt(Number.NEGATIVE_INFINITY)) );

    var MYOB = new MyObject(1234567890);
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(0)",  "1",        (MYOB.charAt(0) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(1)",  "2",        (MYOB.charAt(1) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(2)",  "3",        (MYOB.charAt(2) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(3)",  "4",        (MYOB.charAt(3) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(4)",  "5",        (MYOB.charAt(4) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(5)",  "6",        (MYOB.charAt(5) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(6)",  "7",        (MYOB.charAt(6) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(7)",  "8",        (MYOB.charAt(7) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(8)",  "9",        (MYOB.charAt(8) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(9)",  "0",        (MYOB.charAt(9) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(10)",  "",       (MYOB.charAt(10) ) );

    array[item++] = Assert.expectEq(       "var MYOB = new MyObject(1234567890), MYOB.charAt(Math.PI)",  "4",        (MYOB = new MyObject(1234567890), MYOB.charAt(Math.PI) ) );

    // MyOtherObject.toString will return "[object Object]

    var MYOB = new MyOtherObject(1234567890);
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(0)",  "[",        (MYOB.charAt(0) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(1)",  "o",        (MYOB.charAt(1) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(2)",  "b",        (MYOB.charAt(2) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(3)",  "j",        (MYOB.charAt(3) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(4)",  "e",        (MYOB.charAt(4) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(5)",  "c",        (MYOB.charAt(5) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(6)",  "t",        (MYOB.charAt(6) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(7)",  " ",        (MYOB.charAt(7) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(8)",  "O",        (MYOB.charAt(8) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(9)",  "b",        (MYOB.charAt(9) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(10)",  "j",        (MYOB.charAt(10) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(11)",  "e",        (MYOB.charAt(11) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(12)",  "c",        (MYOB.charAt(12) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(13)",  "t",        (MYOB.charAt(13) ) );
    array[item++] = Assert.expectEq(       "var MYOB = new MyOtherObject(1234567890), MYOB.charAt(14)",  "]",        (MYOB.charAt(14) ) );

    return (array );
}

function MyObject( value ) {
    this.value      = value;
    this.valueOf    = function() { return this.value; }
    this.toString   = function() { return this.value +'' }
    this.charAt     = String.prototype.charAt;
}
function MyOtherObject(value) {
    this.value      = value;
    this.valueOf    = function() { return this.value; }
    this.charAt     = String.prototype.charAt;
}
