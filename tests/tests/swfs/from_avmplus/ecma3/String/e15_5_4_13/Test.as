/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.5.4.13";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.slice( start, end )";
    

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq(   "String.prototype.slice.length",        2,          String.prototype.slice.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.slice.length", false,      delete String.prototype.slice.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.slice.length; String.prototype.slice.length", 2,      (delete String.prototype.slice.length, String.prototype.slice.length) );

    // test cases for when slice is called with no arguments.

    // this is a string object

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); typeof s.slice()",
                                    "string",
                                    (s = new String('this is a string object'), typeof s.slice() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(''); s.slice(1,0)",
                                    "",
                                    (s = new String(''), s.slice(1,0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(true, false)",
                                    "t",
                                    (s = new String('this is a string object'), s.slice(false, true) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(NaN, Infinity)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.slice(NaN, Infinity) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(Infinity, NaN)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(Infinity, NaN) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(Infinity, Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(Infinity, Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(-0.01, 0)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(-0.01,0) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(s.length, s.length)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(s.length, s.length) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(s.length+1, 0)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(s.length+1, 0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.slice(-Infinity, -Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.slice(-Infinity, -Infinity) ) );

    


    // this is an object object
    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.slice = String.prototype.slice; obj.slice(0,8)",
                                    "[object ",
                                    (obj = new Object(), obj.toString= Object.prototype.toString,obj.slice = String.prototype.slice, obj.slice(0,8) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.slice = String.prototype.slice; obj.slice(8,obj.toString().length)",
                                    "Object]",
                                    (obj = new Object(), obj.slice = String.prototype.slice, obj.slice(8, obj.toString().length) ) );


    // this is a function object
    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.slice = Object.prototype.slice; obj.slice(8, Infinity)",
                                    " Function() {}",
                                    (obj = function() {}, obj.slice = String.prototype.slice,obj.slice(8,Infinity)+"" ) );

  


    // this is a user defined object
    array[item++] = Assert.expectEq( 
                                    "var obj = new MyObject( void 0 ); obj.slice(0, 100)",
                                    "undefined",
                                    (obj = new MyObject( void 0 ), obj.slice(0,100) ));

    return array;
}
function MyObject( value ) {
    this.value = value;
    this.slice= String.prototype.slice;
    this.substring = String.prototype.substring;
    this.toString = function() { return this.value+''; }
}

