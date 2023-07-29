/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "substr";
//     var VERSION = "AS3.0";
//     var TITLE   = "String.substr( start, end )";
    

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq(   "String.prototype.substr.length",        2,          String.prototype.substr.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.substr.length", false,      delete String.prototype.substr.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.substr.length; String.prototype.substr.length", 2,      (delete String.prototype.substr.length, String.prototype.substr.length) );

    // test cases for when substr is called with no arguments.

    // this is a string object

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); typeof s.substr()",
                                    "string",
                                    (s = new String('this is a string object'), typeof s.substr() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(''); s.substr(1,0)",
                                    "",
                                    (s = new String(''), s.substr(1,0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(true, false)",
                                    "t",
                                    (s = new String('this is a string object'), s.substr(false, true) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(NaN, Infinity)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substr(NaN, Infinity) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(Infinity, NaN)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(Infinity, NaN) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(Infinity, Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(Infinity, Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(-0.01, 0)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(-0.01,0) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(s.length, s.length)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(s.length, s.length) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(s.length+1, 0)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(s.length+1, 0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(-Infinity, -Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(-Infinity, -Infinity) ) );

   
    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(NaN)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substr(NaN) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(-0.01)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substr(-0.01) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(s.length)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(s.length) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(s.length+1)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(s.length+1) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substr(Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substr(-Infinity)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substr(-Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substr = String.prototype.substr; obj.substr(0,8)",
                                    "[object ",
                                    (obj = new Object(), obj.substr = String.prototype.substr, obj.substr(0,8) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substr = String.prototype.substr; obj.substr(8,obj.toString().length)",
                                    "Object]",
                                    (obj = new Object(), obj.substr = String.prototype.substr, obj.substr(8, obj.toString().length) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.substr = String.prototype.substr; obj.substr(8, Infinity)",
                                    " Function() {}",
                                    (obj = function() {}, obj.substr= String.prototype.substr, obj.substr(8,Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substr = String.prototype.substr; obj.substr(8)",
                                    "Object]",
                                    (obj = new Object(), obj.substr = String.prototype.substr, obj.substr(8) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.substr = String.prototype.substr; obj.substr(8)",
                                    " Function() {}",
                                    (obj = function() {}, obj.substr = String.prototype.substr, obj.substr(8) ) );


    return array;
}

function MyObject( value ) {
    this.value = value;
    this.substring = String.prototype.substring;
   this.toString = function() { return this.value+''; }
}

