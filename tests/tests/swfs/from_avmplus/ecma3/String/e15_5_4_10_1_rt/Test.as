/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/*
//s = new String('this is a string object');
//s.substring(Infinity, Infinity)
a = -22;
b = -a;
b = 0 - a;
x = -Infinity;
//y = Infinity;
*/

//     var SECTION = "15.5.4.10-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.substring( start, end )";
    

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq(   "String.prototype.substring.length",        2,          String.prototype.substring.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.substring.length", false,      delete String.prototype.substring.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.substring.length; String.prototype.substring.length", 2,      (delete String.prototype.substring.length, String.prototype.substring.length) );

    // test cases for when substring is called with no arguments.

    // this is a string object

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); typeof s.substring()",
                                    "string",
                                    (s = new String('this is a string object'), typeof s.substring() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(''); s.substring(1,0)",
                                    "",
                                    (s = new String(''), s.substring(1,0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(true, false)",
                                    "t",
                                    (s = new String('this is a string object'), s.substring(false, true) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(NaN, Infinity)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(NaN, Infinity) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(Infinity, NaN)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(Infinity, NaN) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(Infinity, Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(Infinity, Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(-0.01, 0)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(-0.01,0) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(s.length, s.length)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(s.length, s.length) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(s.length+1, 0)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(s.length+1, 0) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(-Infinity, -Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(-Infinity, -Infinity) ) );

    // this is not a String object, start is not an integer


    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring(Infinity,-Infinity)",
                                    "1,2,3,4,5",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring(Infinity,-Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring(9,-Infinity)",
                                    "1,2,3,4,5",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring(9,-Infinity) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring(true, false)",
                                    "1",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring(true, false) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring('4', '5')",
                                    "3",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring('4', '5') ) );


    // this is an object object
    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substring = String.prototype.substring; obj.substring(8,0)",
                                    "[object ",
                                    (obj = new Object(), obj.substring = String.prototype.substring, obj.substring(8,0) ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substring = String.prototype.substring; obj.substring(8,obj.toString().length)",
                                    "Object]",
                                    (obj = new Object(), obj.substring = String.prototype.substring, obj.substring(8, obj.toString().length) ) );


    // this is a function object
   /* array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.substring = String.prototype.substring; obj.toString = Object.prototype.toString; obj.substring(8, Infinity)",
                                    "Function]",
                                    (obj = function() {}, obj.substring = String.prototype.substring,obj.toString = Object.prototype.toString; obj.substring(8,Infinity) ) );*/

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.substring = String.prototype.substring; obj.toString = Object.prototype.toString; obj.substring(8, Infinity)",
                                    " Function() {}",
                                    (obj = function() {}, obj.substring = String.prototype.substring, obj.substring(8,Infinity) ) );

    // this is a number object
    thisError="no error";
    try{
        var obj = new Number(NaN);
        obj.substring = String.prototype.substring;
        obj.substring(Infinity, NaN);
    }catch(e2:Error){
        thisError=e2.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var obj = new Number(NaN); obj.substring = String.prototype.substring; obj.substring(Infinity, NaN)",
                                    "ReferenceError: Error #1056",
                                    thisError.substring(0,27) );
    }
    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Number(NaN); obj.substring = String.prototype.substring; obj.substring(Infinity, NaN)",
                                    "NaN",
                                    (obj = new Number(NaN), obj.substring = String.prototype.substring, obj.substring(Infinity, NaN) ) );*/


    // this is the Math object
    array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.substring = String.prototype.substring; obj.substring(Math.PI, -10)",
                                    "[cl",
                                    (obj = Math, obj.substring = String.prototype.substring, obj.substring(Math.PI, -10) ) );

    // this is a Boolean object
    thisError="no error";
    try{
        var obj = new Boolean();
        obj.substring = String.prototype.substring;
        obj.substring(new Array(), new Boolean(1));
    }catch(e:Error){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.substring = String.prototype.substring; obj.substring(new Array(), new Boolean(1))",
                                    "ReferenceError: Error #1056",
                                    thisError.substring(0,27) );
    }


    
    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.substring = String.prototype.substring; obj.substring(new Array(), new Boolean(1))",
                                    "f",
                                    (obj = new Boolean(), obj.substring = String.prototype.substring, obj.substring(new Array(), new Boolean(1)) ) );*/

    // this is a user defined object
    array[item++] = Assert.expectEq( 
                                    "var obj = new MyObject( void 0 ); obj.substring(0, 100)",
                                    "undefined",
                                    (obj = new MyObject( void 0 ), obj.substring(0,100) ));

    return array;
}

function MyObject( value ) {
    this.value = value;
    this.substring = String.prototype.substring;
   this.toString = function() { return this.value+''; }
}

