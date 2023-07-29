/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_6_1_3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // tests for boolean primitive, boolean object, Object object, a "MyObject" whose value is
    // a boolean primitive and a boolean object,

    var DATE1 = new Date(0);

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + DATE1",
                                    DATE1.toString() + DATE1.toString(),
                                    DATE1 + DATE1 );

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + 0",
                                    DATE1.toString() + 0,
                                    DATE1 + 0 );

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + new Number(0)",
                                    DATE1.toString() + 0,
                                    DATE1 + new Number(0) );

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + true",
                                    DATE1.toString() + "true",
                                    DATE1 + true );

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + new Boolean(true)",
                                    DATE1.toString() + "true",
                                    DATE1 + new Boolean(true) );

    array[item++] = Assert.expectEq(   
                                    "var DATE1 = new Date(); DATE1 + new Boolean(true)",
                                    DATE1.toString() + "true",
                                    DATE1 + new Boolean(true) );

    var MYOB1 = new MyObject( DATE1 );
    var MYOB2 = new MyValuelessObject( DATE1 );
    //var MYOB3 = new MyProtolessObject( DATE1 );
    //var MYOB4 = new MyProtoValuelessObject( DATE1 );

    array[item++] = Assert.expectEq(   
                                    "MYOB1 = new MyObject(DATE1); MYOB1 + new Number(1)",
                                    "[object Object]1",
                                    MYOB1 + new Number(1) );

    array[item++] = Assert.expectEq(   
                                    "MYOB1 = new MyObject(DATE1); MYOB1 + 1",
                                    "[object Object]1",
                                    MYOB1 + 1 );

/*    array[item++] = Assert.expectEq(   
                                    "MYOB2 = new MyValuelessObject(DATE1); MYOB3 + 'string'",
                                    DATE1.toString() + "string",
                                    MYOB2 + 'string' );

    array[item++] = Assert.expectEq(   
                                    "MYOB2 = new MyValuelessObject(DATE1); MYOB3 + new String('string')",
                                    DATE1.toString() + "string",
                                    MYOB2 + new String('string') );

    array[item++] = Assert.expectEq(   
                                    "MYOB3 = new MyProtolessObject(DATE1); MYOB3 + new Boolean(true)",
                                    DATE1.toString() + "true",
                                    MYOB3 + new Boolean(true) );
*/
    array[item++] = Assert.expectEq(   
                                    "MYOB1 = new MyObject(DATE1); MYOB1 + true",
                                    "[object Object]true",
                                    MYOB1 + true );

    return ( array );
}

/*  cn:  __proto__ is not ecma3 compliant

function MyProtoValuelessObject() {
//this.valueOf = new Function ( "" );
    this.valueOf = function (){ ""};
    this.__proto__ = null;
}
function MyProtolessObject( value ) {
//    this.valueOf = new Function( "return this.value" );
    this.valueOf = function(){return this.value};
    this.__proto__ = null;
    this.value = value;
}
*/
function MyValuelessObject(value) {
    //this.__proto__ = new MyPrototypeObject(value);
    this.constructor.prototype = new MyPrototypeObject(value);
}
function MyPrototypeObject(value) {
    this.valueOf = function(){return this.value};
    this.toString = function(){return this.value + ''};
    this.value = value;
}
function MyObject( value ) {
    this.valueOf = function(){return this.value};
    this.value = value;
}
