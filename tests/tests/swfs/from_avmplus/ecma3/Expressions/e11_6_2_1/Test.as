/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_6_2_1";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // tests for boolean primitive, boolean object, Object object, a "MyObject" whose value is
    // a boolean primitive and a boolean object, and "MyValuelessObject", where the value is
    // set in the object's prototype, not the object itself.
    var EXP_1 = true; var EXP_2 = false;
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = true; var EXP_2 = false; EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Boolean(true); var EXP_2 = new Boolean(false);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Boolean(true); var EXP_2 = new Boolean(false); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Object(true); var EXP_2 = new Object(false);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(true); var EXP_2 = new Object(false); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Object(new Boolean(true)); var EXP_2 = new Object(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(new Boolean(true)); var EXP_2 = new Object(new Boolean(false)); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyObject(true); var EXP_2 = new MyObject(false);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(true); var EXP_2 = new MyObject(false); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyObject(new Boolean(true)); var EXP_2 = new MyObject(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(new Boolean(true)); var EXP_2 = new MyObject(new Boolean(false)); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyOtherObject(new Boolean(true)); var EXP_2 = new MyOtherObject(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyOtherObject(new Boolean(true)); var EXP_2 = new MyOtherObject(new Boolean(false)); EXP_1 - EXP_2",
                                    1,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyValuelessObject(true);
    var EXP_2 = new MyValuelessObject(false);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyValuelessObject(true); var EXP_2 = new MyValuelessObject(false); EXP_1 - EXP_2",
                                    NaN,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyValuelessObject(new Boolean(true));
    var EXP_2 = new MyValuelessObject(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyValuelessObject(new Boolean(true)); var EXP_2 = new MyValuelessObject(new Boolean(false)); EXP_1 - EXP_2",
                                    0, // true - true = 0 not NaN
                                     EXP_1 - EXP_2 );

    // tests for number primitive, number object, Object object, a "MyObject" whose value is
    // a number primitive and a number object, and "MyValuelessObject", where the value is
    // set in the object's prototype, not the object itself.
    var EXP_1 = 100; var EXP_2 = 1;
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = 100; var EXP_2 = 1; EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Number(100);
    var EXP_2 = new Number(1);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Number(100); var EXP_2 = new Number(1); EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Object(100);
    var EXP_2 = new Object(1);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(100); var EXP_2 = new Object(1); EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new Object(new Number(100)); var EXP_2 = new Object(new Number(1));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(new Number(100)); var EXP_2 = new Object(new Number(1)); EXP_1 - EXP_2",
                                    99,
                                      EXP_1 - EXP_2 );
    var EXP_1 = new MyObject(100); var EXP_2 = new MyObject(1);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(100); var EXP_2 = new MyObject(1); EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyObject(new Number(100));
    var EXP_2 = new MyObject(new Number(1));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(new Number(100)); var EXP_2 = new MyObject(new Number(1)); EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyOtherObject(new Number(100)); var EXP_2 = new MyOtherObject(new Number(1));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyOtherObject(new Number(100)); var EXP_2 = new MyOtherObject(new Number(1)); EXP_1 - EXP_2",
                                    99,
                                     EXP_1 - EXP_2 );
    var EXP_1 = new MyValuelessObject(100);
    var EXP_2 = new MyValuelessObject(1);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyValuelessObject(100); var EXP_2 = new MyValuelessObject(1); EXP_1 - EXP_2",
                                    0,
                                     EXP_1 - EXP_2 );
    // same thing with string!
    var EXP_1 = new MyOtherObject(new String('0xff')); var EXP_2 = new MyOtherObject(new String('1'));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyOtherObject(new String('0xff')); var EXP_2 = new MyOtherObject(new String('1'); EXP_1 - EXP_2",
                                    254,
                                     EXP_1 - EXP_2 );

    return ( array );
}
function MyProtoValuelessObject() {
    this.valueOf = function (){ "" };
    this.constructor.prototype = null;
}
function MyProtolessObject( value ) {
    this.valueOf = function(){return this.value};
    this.constructor.prototype= null;
    this.value = value;
}
function MyValuelessObject(value) {
    this.constructor.prototype= new MyPrototypeObject(value);
}
function MyPrototypeObject(value) {
    this.valueOf = function(){return this.value};
    this.toString = function(){return this.value + ''};
    this.value = value;
}
function MyObject( value ) {
    this.valueOf = function( ){return this.value};
    this.value = value;
}
function MyOtherObject( value ) {
    this.valueOf = function( ) { return this.value; };
    this.toString = function( ) { return this.value + ''; };
    this.value = value;
}
