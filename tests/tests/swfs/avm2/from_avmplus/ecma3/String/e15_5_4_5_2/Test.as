/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.5-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charCodeAt";


    var TEST_STRING = new String( " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~" );

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    
    var x;

    var origBooleanCharCodeAt = Boolean.prototype.charCodeAt;
    Boolean.prototype.charCodeAt=String.prototype.charCodeAt;

    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(0)",
            0x0074,
            (x = new Boolean(true), x.charCodeAt(0)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(1)",
            0x0072,
            (x = new Boolean(true), x.charCodeAt(1)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(2)",
            0x0075,
            (x = new Boolean(true), x.charCodeAt(2)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(3)",
            0x0065,
            (x = new Boolean(true), x.charCodeAt(3)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(4)",
            Number.NaN,
            (x = new Boolean(true), x.charCodeAt(4)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(-1)",
            Number.NaN,
            (x = new Boolean(true), x.charCodeAt(-1)) );

    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(true)",
            0x0072,
            (x = new Boolean(true), x.charCodeAt(true)) );
    array[item++] = Assert.expectEq( 
            "x = new Boolean(true); x.charCodeAt=String.prototype.charCodeAt;x.charCodeAt(false)",
            0x0074,
            (x = new Boolean(true), x.charCodeAt(false)) );

    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(0)",    Number.NaN,     (x=new String(),x.charCodeAt(0)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(1)",    Number.NaN,     (x=new String(),x.charCodeAt(1)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(-1)",   Number.NaN,     (x=new String(),x.charCodeAt(-1)) );

    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(NaN)",                       Number.NaN,     (x=new String(),x.charCodeAt(Number.NaN)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(Number.POSITIVE_INFINITY)",  Number.NaN,     (x=new String(),x.charCodeAt(Number.POSITIVE_INFINITY)) );
    array[item++] = Assert.expectEq(      "x = new String(); x.charCodeAt(Number.NEGATIVE_INFINITY)",  Number.NaN,     (x=new String(),x.charCodeAt(Number.NEGATIVE_INFINITY)) );

    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(0)",    0x0031,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(0)) );
    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(1)",    0x002C,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(1)) );
    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(2)",    0x0032,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(2)) );
    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(3)",    0x002C,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(3)) );
    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(4)",    0x0033,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(4)) );
    array[item++] = Assert.expectEq(   "x = new Array(1,2,3); x.charCodeAt = String.prototype.charCodeAt; x.charCodeAt(5)",    NaN,   (x = new Array(1,2,3), x.charCodeAt = String.prototype.charCodeAt, x.charCodeAt(5)) );

    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(0)", 0x005B, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(0)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(1)", 0x006F, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(1)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(2)", 0x0062, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(2)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(3)", 0x006A, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(3)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(4)", 0x0065, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(4)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(5)", 0x0063, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(5)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(6)", 0x0074, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(6)) );

    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(7)", 0x0020, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(7)) );

    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(8)", 0x004F, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(8)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(9)", 0x0062, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(9)) );
    array[item++] = Assert.expectEq(   "x = function() { this.charCodeAt = String.prototype.charCodeAt }; f = new x(); f.charCodeAt(10)", 0x006A, (x = function() { this.charCodeAt = String.prototype.charCodeAt; }, f = new x(), f.charCodeAt(10)) );

    //restore
    Boolean.prototype.charCodeAt = origBooleanCharCodeAt;
    
    return (array );
}

