/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// var SECTION = "Function";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "ReturnTypes";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//vars, functions and classes to be used by the test
import functionReturnTypes.*;

import com.adobe.test.Assert;
var myClassA:TestClassA = new TestClassA(); // class TestClassA {}
var myClassB:TestClassB = new TestClassB(); // class TestClassB extends TestClassA {}
var myClassC:TestClassC = new TestClassC(); // class TestClassC extends TestClassB implements TestInterface {}
                                            // interface TestInterface {}
function emptyFunction() {};
var emptyObject:Object = new Object();
var myDate:Date = new Date(1977,8,24);


var valueArr:Array = [{},"string","10",null,undefined,true,false,0,1,-1,1.23,-1.23,NaN,Infinity,emptyFunction,emptyObject,
            myClassA, myClassB,myClassC,myDate,Number.MAX_VALUE,Number.MIN_VALUE,Number.NEGATIVE_INFINITY,Number.POSITIVE_INFINITY,
            uint.MAX_VALUE,uint.MIN_VALUE,int.MAX_VALUE,int.MIN_VALUE,""];

// The valueDescArr array has the string representations of the valueArr values.
// This is due to the fact that some values (objects) do not resolve to strings.
var valueDescArr:Array = ["{}",'"string"','"10"',"null","undefined","true","false","0","1","-1","1.23","-1.23","NaN","Infinity","emptyFunction",
            "emptyObject","myClassA","myClassB","myClassC","myDate","Number.MAX_VALUE",'Number.MIN_VALUE','Number.NEGATIVE_INFINITY',
            'Number.POSITIVE_INFINITY','uint.MAX_VALUE','uint.MIN_VALUE','int.MAX_VALUE','int.MIN_VALUE','"" (empty string)'];


var typeArr:Array =[String,Number,int,uint,Boolean,Object,Function,TestClassA,TestClassB,TestClassC,Date];
var typeDescArr:Array = ["String","Number","int","uint","Boolean","Object","Function","TestClassA","TestClassB","TestClassC","Date"];



function returnArg(arg) {return arg;}

var funcArr:Array = new Array();

funcArr[0]  = function (arg):String {return arg;}
funcArr[1]  = function (arg):Number {return arg;}
funcArr[2]  = function (arg):int {return arg;}
funcArr[3]  = function (arg):uint {return arg;}
funcArr[4]  = function (arg):Boolean {return arg;}
funcArr[5]  = function (arg):Object {return arg;}
funcArr[6]  = function (arg):Function {return arg;}
funcArr[7]  = function (arg):TestClassA {return arg;}
funcArr[8]  = function (arg):TestClassB {return arg;}
funcArr[9]  = function (arg):TestClassC {return arg;}
funcArr[10] = function (arg):Date {return arg;}

var result;
var resultArr:Array = new Array();
populateResults();

var resultCounter = 0;
//Error cases that try to return an incorrect return type.
for (var i=0; i<funcArr.length; i++) {  //loop through each function
    for (var j=0; j<valueArr.length; j++) { //loop through each type
    try {
        result = funcArr[i](valueArr[j]) is typeArr[i];
    } catch (e) {
        result = e;
    } finally {
       Assert.expectEq("return type: "+ typeDescArr[i] + " arg: "+valueDescArr[j], String(resultArr[resultCounter++]), String(result).substr(0,22));
       //trace("resultArr[x++] = "+result+"; // funcArr["+i+"]("+valueDescArr[j]+") - return type: "+typeDescArr[i]  );
    }
    }
}





              // displays results.

function populateResults() {
var x=0;
resultArr[x++] = true; // funcArr[0]({}) - return type: String
resultArr[x++] = true; // funcArr[0]("string") - return type: String
resultArr[x++] = true; // funcArr[0]("10") - return type: String
resultArr[x++] = false; // funcArr[0](null) - return type: String
resultArr[x++] = false; // funcArr[0](undefined) - return type: String
resultArr[x++] = true; // funcArr[0](true) - return type: String
resultArr[x++] = true; // funcArr[0](false) - return type: String
resultArr[x++] = true; // funcArr[0](0) - return type: String
resultArr[x++] = true; // funcArr[0](1) - return type: String
resultArr[x++] = true; // funcArr[0](-1) - return type: String
resultArr[x++] = true; // funcArr[0](1.23) - return type: String
resultArr[x++] = true; // funcArr[0](-1.23) - return type: String
resultArr[x++] = true; // funcArr[0](NaN) - return type: String
resultArr[x++] = true; // funcArr[0](Infinity) - return type: String
resultArr[x++] = true; // funcArr[0](emptyFunction) - return type: String
resultArr[x++] = true; // funcArr[0](emptyObject) - return type: String
resultArr[x++] = true; // funcArr[0](myClassA) - return type: String
resultArr[x++] = true; // funcArr[0](myClassB) - return type: String
resultArr[x++] = true; // funcArr[0](myClassC) - return type: String
resultArr[x++] = true; // funcArr[0](myDate) - return type: String
resultArr[x++] = true; // funcArr[0](Number.MAX_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0](Number.MIN_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0](Number.NEGATIVE_INFINITY) - return type: String
resultArr[x++] = true; // funcArr[0](Number.POSITIVE_INFINITY) - return type: String
resultArr[x++] = true; // funcArr[0](uint.MAX_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0](uint.MIN_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0](int.MAX_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0](int.MIN_VALUE) - return type: String
resultArr[x++] = true; // funcArr[0]("" (empty string)) - return type: String
resultArr[x++] = true; // funcArr[1]({}) - return type: Number
resultArr[x++] = true; // funcArr[1]("string") - return type: Number
resultArr[x++] = true; // funcArr[1]("10") - return type: Number
resultArr[x++] = true; // funcArr[1](null) - return type: Number
resultArr[x++] = true; // funcArr[1](undefined) - return type: Number
resultArr[x++] = true; // funcArr[1](true) - return type: Number
resultArr[x++] = true; // funcArr[1](false) - return type: Number
resultArr[x++] = true; // funcArr[1](0) - return type: Number
resultArr[x++] = true; // funcArr[1](1) - return type: Number
resultArr[x++] = true; // funcArr[1](-1) - return type: Number
resultArr[x++] = true; // funcArr[1](1.23) - return type: Number
resultArr[x++] = true; // funcArr[1](-1.23) - return type: Number
resultArr[x++] = true; // funcArr[1](NaN) - return type: Number
resultArr[x++] = true; // funcArr[1](Infinity) - return type: Number
resultArr[x++] = true; // funcArr[1](emptyFunction) - return type: Number
resultArr[x++] = true; // funcArr[1](emptyObject) - return type: Number
resultArr[x++] = true; // funcArr[1](myClassA) - return type: Number
resultArr[x++] = true; // funcArr[1](myClassB) - return type: Number
resultArr[x++] = true; // funcArr[1](myClassC) - return type: Number
resultArr[x++] = true; // funcArr[1](myDate) - return type: Number
resultArr[x++] = true; // funcArr[1](Number.MAX_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1](Number.MIN_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1](Number.NEGATIVE_INFINITY) - return type: Number
resultArr[x++] = true; // funcArr[1](Number.POSITIVE_INFINITY) - return type: Number
resultArr[x++] = true; // funcArr[1](uint.MAX_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1](uint.MIN_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1](int.MAX_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1](int.MIN_VALUE) - return type: Number
resultArr[x++] = true; // funcArr[1]("" (empty string)) - return type: Number
resultArr[x++] = true; // funcArr[2]({}) - return type: int
resultArr[x++] = true; // funcArr[2]("string") - return type: int
resultArr[x++] = true; // funcArr[2]("10") - return type: int
resultArr[x++] = true; // funcArr[2](null) - return type: int
resultArr[x++] = true; // funcArr[2](undefined) - return type: int
resultArr[x++] = true; // funcArr[2](true) - return type: int
resultArr[x++] = true; // funcArr[2](false) - return type: int
resultArr[x++] = true; // funcArr[2](0) - return type: int
resultArr[x++] = true; // funcArr[2](1) - return type: int
resultArr[x++] = true; // funcArr[2](-1) - return type: int
resultArr[x++] = true; // funcArr[2](1.23) - return type: int
resultArr[x++] = true; // funcArr[2](-1.23) - return type: int
resultArr[x++] = true; // funcArr[2](NaN) - return type: int
resultArr[x++] = true; // funcArr[2](Infinity) - return type: int
resultArr[x++] = true; // funcArr[2](emptyFunction) - return type: int
resultArr[x++] = true; // funcArr[2](emptyObject) - return type: int
resultArr[x++] = true; // funcArr[2](myClassA) - return type: int
resultArr[x++] = true; // funcArr[2](myClassB) - return type: int
resultArr[x++] = true; // funcArr[2](myClassC) - return type: int
resultArr[x++] = true; // funcArr[2](myDate) - return type: int
resultArr[x++] = true; // funcArr[2](Number.MAX_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2](Number.MIN_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2](Number.NEGATIVE_INFINITY) - return type: int
resultArr[x++] = true; // funcArr[2](Number.POSITIVE_INFINITY) - return type: int
resultArr[x++] = true; // funcArr[2](uint.MAX_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2](uint.MIN_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2](int.MAX_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2](int.MIN_VALUE) - return type: int
resultArr[x++] = true; // funcArr[2]("" (empty string)) - return type: int
resultArr[x++] = true; // funcArr[3]({}) - return type: uint
resultArr[x++] = true; // funcArr[3]("string") - return type: uint
resultArr[x++] = true; // funcArr[3]("10") - return type: uint
resultArr[x++] = true; // funcArr[3](null) - return type: uint
resultArr[x++] = true; // funcArr[3](undefined) - return type: uint
resultArr[x++] = true; // funcArr[3](true) - return type: uint
resultArr[x++] = true; // funcArr[3](false) - return type: uint
resultArr[x++] = true; // funcArr[3](0) - return type: uint
resultArr[x++] = true; // funcArr[3](1) - return type: uint
resultArr[x++] = true; // funcArr[3](-1) - return type: uint
resultArr[x++] = true; // funcArr[3](1.23) - return type: uint
resultArr[x++] = true; // funcArr[3](-1.23) - return type: uint
resultArr[x++] = true; // funcArr[3](NaN) - return type: uint
resultArr[x++] = true; // funcArr[3](Infinity) - return type: uint
resultArr[x++] = true; // funcArr[3](emptyFunction) - return type: uint
resultArr[x++] = true; // funcArr[3](emptyObject) - return type: uint
resultArr[x++] = true; // funcArr[3](myClassA) - return type: uint
resultArr[x++] = true; // funcArr[3](myClassB) - return type: uint
resultArr[x++] = true; // funcArr[3](myClassC) - return type: uint
resultArr[x++] = true; // funcArr[3](myDate) - return type: uint
resultArr[x++] = true; // funcArr[3](Number.MAX_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3](Number.MIN_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3](Number.NEGATIVE_INFINITY) - return type: uint
resultArr[x++] = true; // funcArr[3](Number.POSITIVE_INFINITY) - return type: uint
resultArr[x++] = true; // funcArr[3](uint.MAX_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3](uint.MIN_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3](int.MAX_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3](int.MIN_VALUE) - return type: uint
resultArr[x++] = true; // funcArr[3]("" (empty string)) - return type: uint
resultArr[x++] = true; // funcArr[4]({}) - return type: Boolean
resultArr[x++] = true; // funcArr[4]("string") - return type: Boolean
resultArr[x++] = true; // funcArr[4]("10") - return type: Boolean
resultArr[x++] = true; // funcArr[4](null) - return type: Boolean
resultArr[x++] = true; // funcArr[4](undefined) - return type: Boolean
resultArr[x++] = true; // funcArr[4](true) - return type: Boolean
resultArr[x++] = true; // funcArr[4](false) - return type: Boolean
resultArr[x++] = true; // funcArr[4](0) - return type: Boolean
resultArr[x++] = true; // funcArr[4](1) - return type: Boolean
resultArr[x++] = true; // funcArr[4](-1) - return type: Boolean
resultArr[x++] = true; // funcArr[4](1.23) - return type: Boolean
resultArr[x++] = true; // funcArr[4](-1.23) - return type: Boolean
resultArr[x++] = true; // funcArr[4](NaN) - return type: Boolean
resultArr[x++] = true; // funcArr[4](Infinity) - return type: Boolean
resultArr[x++] = true; // funcArr[4](emptyFunction) - return type: Boolean
resultArr[x++] = true; // funcArr[4](emptyObject) - return type: Boolean
resultArr[x++] = true; // funcArr[4](myClassA) - return type: Boolean
resultArr[x++] = true; // funcArr[4](myClassB) - return type: Boolean
resultArr[x++] = true; // funcArr[4](myClassC) - return type: Boolean
resultArr[x++] = true; // funcArr[4](myDate) - return type: Boolean
resultArr[x++] = true; // funcArr[4](Number.MAX_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4](Number.MIN_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4](Number.NEGATIVE_INFINITY) - return type: Boolean
resultArr[x++] = true; // funcArr[4](Number.POSITIVE_INFINITY) - return type: Boolean
resultArr[x++] = true; // funcArr[4](uint.MAX_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4](uint.MIN_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4](int.MAX_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4](int.MIN_VALUE) - return type: Boolean
resultArr[x++] = true; // funcArr[4]("" (empty string)) - return type: Boolean
resultArr[x++] = true; // funcArr[5]({}) - return type: Object
resultArr[x++] = true; // funcArr[5]("string") - return type: Object
resultArr[x++] = true; // funcArr[5]("10") - return type: Object
resultArr[x++] = false; // funcArr[5](null) - return type: Object
resultArr[x++] = false; // funcArr[5](undefined) - return type: Object
resultArr[x++] = true; // funcArr[5](true) - return type: Object
resultArr[x++] = true; // funcArr[5](false) - return type: Object
resultArr[x++] = true; // funcArr[5](0) - return type: Object
resultArr[x++] = true; // funcArr[5](1) - return type: Object
resultArr[x++] = true; // funcArr[5](-1) - return type: Object
resultArr[x++] = true; // funcArr[5](1.23) - return type: Object
resultArr[x++] = true; // funcArr[5](-1.23) - return type: Object
resultArr[x++] = true; // funcArr[5](NaN) - return type: Object
resultArr[x++] = true; // funcArr[5](Infinity) - return type: Object
resultArr[x++] = true; // funcArr[5](emptyFunction) - return type: Object
resultArr[x++] = true; // funcArr[5](emptyObject) - return type: Object
resultArr[x++] = true; // funcArr[5](myClassA) - return type: Object
resultArr[x++] = true; // funcArr[5](myClassB) - return type: Object
resultArr[x++] = true; // funcArr[5](myClassC) - return type: Object
resultArr[x++] = true; // funcArr[5](myDate) - return type: Object
resultArr[x++] = true; // funcArr[5](Number.MAX_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5](Number.MIN_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5](Number.NEGATIVE_INFINITY) - return type: Object
resultArr[x++] = true; // funcArr[5](Number.POSITIVE_INFINITY) - return type: Object
resultArr[x++] = true; // funcArr[5](uint.MAX_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5](uint.MIN_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5](int.MAX_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5](int.MIN_VALUE) - return type: Object
resultArr[x++] = true; // funcArr[5]("" (empty string)) - return type: Object
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6]({}) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6]("string") - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6]("10") - return type: Function
resultArr[x++] = false; // funcArr[6](null) - return type: Function
resultArr[x++] = false; // funcArr[6](undefined) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](true) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](false) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](0) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](1) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](-1) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](1.23) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](-1.23) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](NaN) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](Infinity) - return type: Function
resultArr[x++] = true; // funcArr[6](emptyFunction) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](emptyObject) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](myClassA) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](myClassB) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](myClassC) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](myDate) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](Number.MAX_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](Number.MIN_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](Number.NEGATIVE_INFINITY) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](Number.POSITIVE_INFINITY) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](uint.MAX_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](uint.MIN_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](int.MAX_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6](int.MIN_VALUE) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[6]("" (empty string)) - return type: Function
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7]({}) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7]("string") - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7]("10") - return type: TestClassA
resultArr[x++] = false; // funcArr[7](null) - return type: TestClassA
resultArr[x++] = false; // funcArr[7](undefined) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](true) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](false) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](0) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](1) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](-1) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](1.23) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](-1.23) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](NaN) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](Infinity) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](emptyFunction) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](emptyObject) - return type: TestClassA
resultArr[x++] = true; // funcArr[7](myClassA) - return type: TestClassA
resultArr[x++] = true; // funcArr[7](myClassB) - return type: TestClassA
resultArr[x++] = true; // funcArr[7](myClassC) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](myDate) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](Number.MAX_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](Number.MIN_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](Number.NEGATIVE_INFINITY) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](Number.POSITIVE_INFINITY) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](uint.MAX_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](uint.MIN_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](int.MAX_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7](int.MIN_VALUE) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[7]("" (empty string)) - return type: TestClassA
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8]({}) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8]("string") - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8]("10") - return type: TestClassB
resultArr[x++] = false; // funcArr[8](null) - return type: TestClassB
resultArr[x++] = false; // funcArr[8](undefined) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](true) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](false) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](0) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](1) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](-1) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](1.23) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](-1.23) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](NaN) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](Infinity) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](emptyFunction) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](emptyObject) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](myClassA) - return type: TestClassB
resultArr[x++] = true; // funcArr[8](myClassB) - return type: TestClassB
resultArr[x++] = true; // funcArr[8](myClassC) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](myDate) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](Number.MAX_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](Number.MIN_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](Number.NEGATIVE_INFINITY) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](Number.POSITIVE_INFINITY) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](uint.MAX_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](uint.MIN_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](int.MAX_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8](int.MIN_VALUE) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[8]("" (empty string)) - return type: TestClassB
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9]({}) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9]("string") - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9]("10") - return type: TestClassC
resultArr[x++] = false; // funcArr[9](null) - return type: TestClassC
resultArr[x++] = false; // funcArr[9](undefined) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](true) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](false) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](0) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](1) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](-1) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](1.23) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](-1.23) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](NaN) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](Infinity) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](emptyFunction) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](emptyObject) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](myClassA) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](myClassB) - return type: TestClassC
resultArr[x++] = true; // funcArr[9](myClassC) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](myDate) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](Number.MAX_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](Number.MIN_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](Number.NEGATIVE_INFINITY) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](Number.POSITIVE_INFINITY) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](uint.MAX_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](uint.MIN_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](int.MAX_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9](int.MIN_VALUE) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[9]("" (empty string)) - return type: TestClassC
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10]({}) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10]("string") - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10]("10") - return type: Date
resultArr[x++] = false; // funcArr[10](null) - return type: Date
resultArr[x++] = false; // funcArr[10](undefined) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](true) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](false) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](0) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](1) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](-1) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](1.23) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](-1.23) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](NaN) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](Infinity) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](emptyFunction) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](emptyObject) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](myClassA) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](myClassB) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](myClassC) - return type: Date
resultArr[x++] = true; // funcArr[10](myDate) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](Number.MAX_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](Number.MIN_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](Number.NEGATIVE_INFINITY) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](Number.POSITIVE_INFINITY) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](uint.MAX_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](uint.MIN_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](int.MAX_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10](int.MIN_VALUE) - return type: Date
resultArr[x++] = 'TypeError: Error #1034'; // funcArr[10]("" (empty string)) - return type: Date
}
