/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.5.4.9-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.substring( start )";


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
                                    "var s = new String(''); s.substring()",
                                    "",
                                    (s = new String(''), s.substring() ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring()",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(NaN)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(NaN) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(-0.01)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(-0.01) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(s.length)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(s.length) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(s.length+1)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(s.length+1) ) );


    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(Infinity)",
                                    "",
                                    (s = new String('this is a string object'), s.substring(Infinity) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.substring(-Infinity)",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.substring(-Infinity) ) );

    // this is not a String object, start is not an integer


    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring()",
                                    "1,2,3,4,5",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring(true)",
                                    ",2,3,4,5",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring(true) ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new Array(1,2,3,4,5); s.substring = String.prototype.substring; s.substring('4')",
                                    "3,4,5",
                                    (s = new Array(1,2,3,4,5), s.substring = String.prototype.substring, s.substring('4') ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new Array(); s.substring = String.prototype.substring; s.substring('4')",
                                    "",
                                    (s = new Array(), s.substring = String.prototype.substring, s.substring('4') ) );

    // this is an object object
    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.substring = String.prototype.substring; obj.substring(8)",
                                    "Object]",
                                    (obj = new Object(), obj.substring = String.prototype.substring, obj.substring(8) ) );

    // this is a function object

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.substring = String.prototype.substring; obj.toString = Object.prototype.toString; obj.substring(8)",
                                    true,
                                    (obj = function() {}, obj.substring = String.prototype.substring, obj.toString = Object.prototype.toString, obj.substring(8)).indexOf("Function-") == 0  ||
                                    (obj = function() {}, obj.substring = String.prototype.substring, obj.toString = Object.prototype.toString, obj.substring(8))=="null]"
                                     );
    // this is a number object

    thisError="no error";
    try{
        var obj = new Number(NaN);
        obj.substring = String.prototype.substring;
        obj.substring(false);
    }catch(e1:Error){
        thisError=e1.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var obj = new Number(NaN); obj.substring = String.prototype.substring; obj.substring(false)",
                                    "ReferenceError: Error #1056",Utils.referenceError(thisError)
                                    );
    }
   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Number(NaN); obj.substring = String.prototype.substring; obj.substring(false)",
                                    "NaN",
                                    (obj = new Number(NaN), obj.substring = String.prototype.substring, obj.substring(false) ) );*/


    // this is the Math object
    array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.substring = String.prototype.substring; obj.substring(Math.PI)",
                                    "ass Math]",
                                    (obj = Math, obj.substring = String.prototype.substring, obj.substring(Math.PI) ) );

    // this is a Boolean object

    thisError="no error";
    try{
        var obj = new Boolean();
        obj.substring = String.prototype.substring;
        obj.substring(new Array());
    }catch(e3:Error){
        thisError=e3.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.substring = String.prototype.substring; obj.substring(new Array())",
                                    "ReferenceError: Error #1056",Utils.referenceError(thisError)
                                    );
    }

    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.substring = String.prototype.substring; obj.substring(new Array())",
                                    "false",
                                    (obj = new Boolean(), obj.substring = String.prototype.substring, obj.substring(new Array()) ) );*/

    // this is a user defined object



    array[item++] = Assert.expectEq( 
                                    "var obj = new MyObject( null ); obj.substring(0)",
                                    "null",
                                    (obj = new MyObject( null ), obj.substring(0) ) );

    return array;
}
function MyObject( value ) {
    this.value = value;
    this.substring = String.prototype.substring;
    this.toString = function () { return this.value+'' }
}
