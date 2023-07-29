/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.5.4.8-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.split";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "String.prototype.split.length",        2,          String.prototype.split.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.split.length", false,      delete String.prototype.split.length );
    array[item++] = Assert.expectEq(   "delete String.prototype.split.length; String.prototype.split.length", 2,      (delete String.prototype.split.length, String.prototype.split.length));

    // test cases for when split is called with no arguments.

    // this is a string object

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); typeof s.split()",
                                    "object",
                                    (s = new String('this is a string object'), typeof s.split() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); Array.prototype.getClass = Object.prototype.toString; (s.split()).getClass()",
                                    "[object Array]",
                                    (s = new String('this is a string object'), Array.prototype.getClass = Object.prototype.toString, (s.split()).getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.split().length",
                                    1,
                                    (s = new String('this is a string object'), s.split().length ) );

 
    array[item++] = Assert.expectEq(   
                                    "var s = new String('this is a string object'); s.split()[0]",
                                    "this is a string object",
                                    (s = new String('this is a string object'), s.split()[0] ) );
 

    // this is an object object
    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = new Object(), obj.split = String.prototype.split, typeof obj.split() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = new Object(), obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = new Object(), obj.split = String.prototype.split, obj.split().length ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Object(); obj.split = String.prototype.split; obj.split()[0]",
                                    "[object Object]",
                                    (obj = new Object(), obj.split = String.prototype.split, obj.split()[0] ) );
   
        

    // this is a function object
    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = function() {}, obj.split = String.prototype.split, typeof obj.split() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = function() {}, obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = function() {}, obj.split = String.prototype.split, obj.split().length ) );

   /* commenting out due to bug 175096
    array[item++] = Assert.expectEq(   
                                    "var obj = function() {}; obj.split = String.prototype.split; obj.toString = Object.prototype.toString; obj.split()[0]",
                                    "[object Function]",
                                    (obj = function() {}, obj.split = String.prototype.split, obj.toString = Object.prototype.toString, (obj.split()[0]).substring(0,16)+"]"));
    */

    // this is a number object

    thisError="no error";
    var obj = new Number(NaN);
    try{
    
        obj.split = String.prototype.split;
    
    }catch(e1:Error){
        thisError=e1.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Number(NaN); obj.split = String.prototype.split; typeof obj.split()",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }



   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Number(NaN); obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = new Number(NaN), obj.split = String.prototype.split, typeof obj.split() ) );*/

    thisError="no error";
    var obj = new Number(NaN);
    try{
    
        obj.split = String.prototype.split;
        Array.prototype.getClass = Object.prototype.toString;
        obj.getClass();
    }catch(e2:Error){
        thisError=e2.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Number(Infinity); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Number(Infinity); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = new Number(Infinity), obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );*/

    thisError="no error";
    var obj = new Number(-1234567890);
    try{
        obj.split = String.prototype.split;
        obj.split().length;
    }catch(e3:Error){
        thisError=e3.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Number(-1234567890); obj.split = String.prototype.split; obj.split().length","ReferenceError: Error #1056",Utils.referenceError(thisError));
    }


    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Number(-1234567890); obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = new Number(-1234567890), obj.split = String.prototype.split, obj.split().length ) );*/

    thisError="no error";
    var obj = new Number(-1e21);
    try{
        obj.split = String.prototype.split;
        obj.split()[0];
    }catch(e4:Error){
        thisError=e4.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Number(-1e21); obj.split = String.prototype.split; obj.split()[0]","ReferenceError: Error #1056",Utils.referenceError(thisError));
    }

    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Number(-1e21); obj.split = String.prototype.split; obj.split()[0]",
                                    "-1e+21",
                                    (obj = new Number(-1e21), obj.split = String.prototype.split, obj.split()[0] ) );*/


    // this is the Math object

    array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = Math, obj.split = String.prototype.split, typeof obj.split() ) );

    thisError="no error";
    var obj = Math;
    try{
        obj.split = String.prototype.split;
        Array.prototype.getClass = Object.prototype.toString;
        obj.getClass();
    }catch(e6:Error){
        thisError=e6.toString();
    }finally{
        array[item++] = Assert.expectEq("var obj = Math; obj.split = String.prototype.split;Array.prototype.getClass = Object.prototype.toString; obj.getClass()","TypeError: Error #1006",Utils.typeError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = Math, obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );*/

    array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = Math, obj.split = String.prototype.split, obj.split().length ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = Math; obj.split = String.prototype.split; obj.split()[0]",
                                    "[class Math]",
                                    (obj = Math, obj.split = String.prototype.split, obj.split()[0] ) );

    // this is an array object
    array[item++] = Assert.expectEq(   
                                    "var obj = new Array(1,2,3,4,5); obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = new Array(1,2,3,4,5), obj.split = String.prototype.split, typeof obj.split() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Array(1,2,3,4,5); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = new Array(1,2,3,4,5), obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Array(1,2,3,4,5); obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = new Array(1,2,3,4,5), obj.split = String.prototype.split, obj.split().length ) );

    array[item++] = Assert.expectEq(   
                                    "var obj = new Array(1,2,3,4,5); obj.split = String.prototype.split; obj.split()[0]",
                                    "1,2,3,4,5",
                                    (obj = new Array(1,2,3,4,5), obj.split = String.prototype.split, obj.split()[0] ) );

    // this is a Boolean object

    thisError="no error";
    var obj = new Boolean();
    try{
    
        obj.split = String.prototype.split;
    
    }catch(e9:Error){
        thisError=e9.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Boolean(); obj.split = String.prototype.split; typeof obj.split()",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }



   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.split = String.prototype.split; typeof obj.split()",
                                    "object",
                                    (obj = new Boolean(), obj.split = String.prototype.split, typeof obj.split() ) );*/

    thisError="no error";
    var obj = new Boolean();
    try{
    
        obj.split = String.prototype.split;
        Array.prototype.getClass = Object.prototype.toString;
        obj.getClass();
    
    }catch(e10:Error){
        thisError=e10.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Boolean(); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }


   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.split = String.prototype.split; Array.prototype.getClass = Object.prototype.toString; obj.getClass()",
                                    "[object Array]",
                                    (obj = new Boolean(), obj.split = String.prototype.split, Array.prototype.getClass = Object.prototype.toString, obj.split().getClass() ) );*/

    thisError="no error";
    var obj = new Boolean();
    try{
    
        obj.split = String.prototype.split;
        obj.split().length;
    
    
    }catch(e11:Error){
        thisError=e11.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Boolean(); obj.split = String.prototype.split;obj.split().length",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.split = String.prototype.split; obj.split().length",
                                    1,
                                    (obj = new Boolean(), obj.split = String.prototype.split, obj.split().length ) );*/

    thisError="no error";
    var obj = new Boolean();
    try{
    
        obj.split = String.prototype.split;
        obj.split()[0];
    
    
    }catch(e12:Error){
        thisError=e12.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq("var obj = new Boolean(); obj.split = String.prototype.split;obj.split()[0]",
"ReferenceError: Error #1056",Utils.referenceError(thisError));
    }

    /*array[item++] = Assert.expectEq(   
                                    "var obj = new Boolean(); obj.split = String.prototype.split; obj.split()[0]",
                                    "false",
                                    (obj = new Boolean(), obj.split = String.prototype.split, obj.split()[0] ) );*/


    //delete extra property created
    delete Array.prototype.getClass;
    
    return array;
}
