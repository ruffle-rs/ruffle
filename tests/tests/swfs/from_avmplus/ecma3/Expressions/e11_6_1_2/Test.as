/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_6_1_2";
//     var VERSION = "ECMA_1";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // tests for boolean primitive, boolean object, Object object, a "MyObject" whose value is
    // a boolean primitive and a boolean object
    var EXP_1 = 'string'; var EXP_2 = false;
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = 'string'; var EXP_2 = false; EXP_1 + EXP_2",
                                    "stringfalse",
                                     EXP_1 + EXP_2 );
    var EXP_1 = true; var EXP_2 = 'string';
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = true; var EXP_2 = 'string'; EXP_1 + EXP_2",
                                    "truestring",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new Boolean(true); var EXP_2 = new String('string');
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Boolean(true); var EXP_2 = new String('string'); EXP_1 + EXP_2",
                                    "truestring",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new Object(true); var EXP_2 = new Object('string');
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(true); var EXP_2 = new Object('string'); EXP_1 + EXP_2",
                                    "truestring",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new Object(new String('string')); var EXP_2 = new Object(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(new String('string')); var EXP_2 = new Object(new Boolean(false)); EXP_1 + EXP_2",
                                    "stringfalse",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new MyObject(true); var EXP_2 = new MyObject('string');
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(true); var EXP_2 = new MyObject('string'); EXP_1 + EXP_2",
                                     "truestring",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new MyObject(new String('string')); var EXP_2 = new MyObject(new Boolean(false));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(new String('string')); var EXP_2 = new MyObject(new Boolean(false)); EXP_1 + EXP_2",
                                    "stringfalse",
                                     EXP_1 + EXP_2 );
    //var EXP_1 = new MyValuelessObject('string'); var EXP_2 = new MyValuelessObject(false);
    //array[item++] = Assert.expectEq(   
    //                                "var EXP_1 = new MyValuelessObject('string'); var EXP_2 = new MyValuelessObject(false); EXP_1 + EXP_2",
    //                                "stringfalse",
    //                                 EXP_1 + EXP_2 );
    //var EXP_1 = new MyValuelessObject(new String('string')); var EXP_2 = new MyValuelessObject(new Boolean(false));
    //array[item++] = Assert.expectEq(   
    //                                "var EXP_1 = new MyValuelessObject(new String('string')); var EXP_2 = new MyValuelessObject(new Boolean(false)); EXP_1 + EXP_2",
    //                                "stringfalse",
    //                                 EXP_1 + EXP_2 );

    // tests for number primitive, number object, Object object, a "MyObject" whose value is
    // a number primitive and a number object
    var EXP_1 = 100; var EXP_2 = 'string';
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = 100; var EXP_2 = 'string'; EXP_1 + EXP_2",
                                    "100string",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new String('string'); var EXP_2 = new Number(-1);
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new String('string'); var EXP_2 = new Number(-1); EXP_1 + EXP_2",
                                    "string-1",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new Object(100); var EXP_2 = new Object('string');
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(100); var EXP_2 = new Object('string'); EXP_1 + EXP_2",
                                    "100string",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new Object(new String('string')); var EXP_2 = new Object(new Number(-1));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new Object(new String('string')); var EXP_2 = new Object(new Number(-1)); EXP_1 + EXP_2",
                                    "string-1",
                                     EXP_1 + EXP_2 );

    var EXP_1 = new MyObject(100); var EXP_2 = new MyObject('string');
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(100); var EXP_2 = new MyObject('string'); EXP_1 + EXP_2",
                                    "100string",
                                     EXP_1 + EXP_2 );
    var EXP_1 = new MyObject(new String('string')); var EXP_2 = new MyObject(new Number(-1));
    array[item++] = Assert.expectEq(   
                                    "var EXP_1 = new MyObject(new String('string')); var EXP_2 = new MyObject(new Number(-1)); EXP_1 + EXP_2",
                                    "string-1",
                                     EXP_1 + EXP_2 );
    //var EXP_1 = new MyValuelessObject(100); var EXP_2 = new MyValuelessObject('string');
    //array[item++] = Assert.expectEq(   
    //                                "var EXP_1 = new MyValuelessObject(100); var EXP_2 = new MyValuelessObject('string'); EXP_1 + EXP_2",
    //                                "100string",
    //                                 EXP_1 + EXP_2 );
    //var EXP_1 = new MyValuelessObject(new String('string')); var EXP_2 = new MyValuelessObject(new Number(-1));
    //array[item++] = Assert.expectEq(   
    //                                "var EXP_1 = new MyValuelessObject(new String('string')); var EXP_2 = new MyValuelessObject(new Number(-1)); EXP_1 + EXP_2",
    //                                "string-1",
    //                                EXP_1 + EXP_2 );
    return ( array );
}

// cn: __proto__ is not ecma3 compliant
/*
function MyProtoValuelessObject() {
//this.valueOf = new Function ( "" );
    this.valueOf = function (){""};
    this.__proto__ = null;
}
function MyProtolessObject( value ) {
//this.valueOf = new Function( "return this.value" );
    this.valueOf = function(){return this.value};
    this.__proto__ = null;
    this.value = value;
}
function MyValuelessObject(value) {
    this.__proto__ = new MyPrototypeObject(value);
}
*/
function MyPrototypeObject(value) {
//    this.valueOf = new Function( "return this.value;" );
 //   this.toString = new Function( "return (this.value + '');" );
    this.valueOf = function(){return this.value};
    this.toString = function(){return this.value + ''};
    this.value = value;
}
function MyObject( value ) {
//this.valueOf = new Function( "return this.value" );
    this.valueOf = function(){return this.value};
    this.value = value;
}
