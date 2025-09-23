/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}
// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "is Operator";       // Provide ECMA section title or a description



//vars, functions and classes to be used by the test
import isOper.*;

import com.adobe.test.Assert;
var myClassA:TestClassA = new TestClassA(); // class TestClassA {}
var myClassB:TestClassB = new TestClassB(); // class TestClassB extends TestClassA {}
var myClassC:TestClassC = new TestClassC(); // class TestClassC extends TestClassB implements TestInterface {}
                                            // interface TestInterface {}
function emptyFunction() {};
var emptyAnyObj:* = new Object();
var undefinedVar;
var emptyObject:Object = new Object();
var myDate:Date = new Date(1977,8,24);
var filledArr:Array = [0,1,2,3,4,5,6,7,"eight",new Object(),Math.PI,9,10];
var emptyArr = [];


var valueArr:Array = [{},"string","10",null,undefined,true,false,0,1,-1,1.23,-1.23,NaN,Infinity,emptyFunction,emptyObject,
            myClassA, myClassB,myClassC,myDate,Number.MAX_VALUE,Number.MIN_VALUE,Number.NEGATIVE_INFINITY,Number.POSITIVE_INFINITY,
            uint.MAX_VALUE,uint.MIN_VALUE,int.MAX_VALUE,int.MIN_VALUE,"",emptyAnyObj,undefinedVar,filledArr,emptyArr];

// The valueDescArr array has the string representations of the valueArr values.
// This is due to the fact that some values (objects) do not resolve to strings.
var valueDescArr:Array = ["{}",'"string"','"10"',"null","undefined","true","false","0","1","-1","1.23","-1.23","NaN","Infinity","emptyFunction",
            "emptyObject","myClassA","myClassB","myClassC","myDate","Number.MAX_VALUE",'Number.MIN_VALUE','Number.NEGATIVE_INFINITY','Number.POSITIVE_INFINITY',
            'uint.MAX_VALUE','uint.MIN_VALUE','int.MAX_VALUE','int.MIN_VALUE','"" (empty string)',"empty * Obj","undefinedVar","filled Array","empty Array"];

var typeArr:Array =     [String,Number,int,uint,Boolean,Object, Function,TestClassA,TestClassB,TestClassC,TestInterface,Date,Array];
var typeDescArr:Array = ["String","Number","int","uint","Boolean","Object","Function","TestClassA","TestClassB","TestClassC","TestInterface","Date","Array"];

// The resultArr Array holds the expected boolean values when each value is compared to type using "is"
var resultArr  = new Array(typeArr.length);

var x:int = 0;  //counter for resultArr.  DO NOT change the line order of the array.

//          [String,Number, int,    uint,   Boolean,Object, Function,TestClassA, TestClassB,TestClassC, TestInterface,  Date,   Array];
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // {}
resultArr[x++] =    [true,  false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // "string"
resultArr[x++] =    [true,  false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // "10"
resultArr[x++] =    [false, false,  false,  false,  false,  false,  false,      false,  false,      false,      false,  false,  false];     // null
resultArr[x++] =    [false, false,  false,  false,  false,  false,  false,      false,  false,      false,      false,  false,  false];     // undefined
resultArr[x++] =    [false, false,  false,  false,  true,   true,   false,      false,  false,      false,      false,  false,  false];     // true
resultArr[x++] =    [false, false,  false,  false,  true,   true,   false,      false,  false,      false,      false,  false,  false];     // false
resultArr[x++] =    [false, true,   true,   true,   false,  true,   false,      false,  false,      false,      false,  false,  false];     // 0
resultArr[x++] =    [false, true,   true,   true,   false,  true,   false,      false,  false,      false,      false,  false,  false];     // 1
resultArr[x++] =    [false, true,   true,   false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // -1
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // 1.23
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // -1.23
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // NaN
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // Infinity
resultArr[x++] =    [false, false,  false,  false,  false,  true,   true,       false,  false,      false,      false,  false,  false];     // emptyFunction
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // emptyObject
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      true,   false,      false,      false,  false,  false];     // myClassA
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      true,   true,       false,      false,  false,  false];     // myClassB
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      true,   true,       true,       true,   false,  false];     // myClassC
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  true,   false];     // myDate
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // Number.MAX_VALUE
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // Number.MIN_VALUE
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // Number.NEGATIVE_INFINITY
resultArr[x++] =    [false, true,   false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // Number.POSITIVE_INFINITY
resultArr[x++] =    [false, true,   false,  true,   false,  true,   false,      false,  false,      false,      false,  false,  false];     // uint.MAX_VALUE
resultArr[x++] =    [false, true,   true,   true,   false,  true,   false,      false,  false,      false,      false,  false,  false];     // uint.MIN_VALUE
resultArr[x++] =    [false, true,   true,   true,   false,  true,   false,      false,  false,      false,      false,  false,  false];     // int.MAX_VALUE
resultArr[x++] =    [false, true,   true,   false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // int.MIN_VALUE
resultArr[x++] =    [true,  false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // "" (empty string)
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  false];     // emptyAnyObj
resultArr[x++] =    [false, false,  false,  false,  false,  false,  false,      false,  false,      false,      false,  false,  false];     // undefinedVar
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  true];      // filledArray
resultArr[x++] =    [false, false,  false,  false,  false,  true,   false,      false,  false,      false,      false,  false,  true];      // emptyArray


var typeArrLength = typeArr.length;

for (var i:int = 0; i < valueArr.length; i++) {
    for (var j:int = 0; j < typeArrLength; j++) {
        Assert.expectEq(valueDescArr[i]+" is "+ typeDescArr[j],resultArr[i][j],(valueArr[i] is typeArr[j]));
    }
}

////////////////////////////////////////////////////////////////

              // displays results.
