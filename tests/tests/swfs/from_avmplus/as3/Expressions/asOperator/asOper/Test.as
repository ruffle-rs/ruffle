/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "as Operator";       // Provide ECMA section title or a description



//Note that this uses the same value and type arrays as expressions/isOperator/isOper.as

//vars, functions and classes to be used by the test
import asOper.*;

import com.adobe.test.Assert;
var myClassA:TestClassA = new TestClassA(); // class TestClassA {}
var myClassB:TestClassB = new TestClassB(); // class TestClassB extends TestClassA {}
var myClassC:TestClassC = new TestClassC(); // class TestClassC extends TestClassB implements TestInterface {}
                                            // interface TestInterface {}
function emptyFunction() {};
var emptyObject:Object = new Object();
var myDate:Date = new Date(1977,8,24);

// https://bugzilla.mozilla.org/show_bug.cgi?id=555805 - MIN_VALUE is not a cross-platform constant.
// Don't use it here.

var valueArr:Array = [{},"string","10",null,undefined,true,false,0,1,-1,1.23,-1.23,NaN,Infinity,emptyFunction,emptyObject,
                      myClassA, myClassB,myClassC,myDate,Number.MAX_VALUE,Number.NEGATIVE_INFINITY,Number.POSITIVE_INFINITY,
                      uint.MAX_VALUE,uint.MIN_VALUE,int.MAX_VALUE,int.MIN_VALUE,""];

// The valueDescArr array has the string representations of the valueArr values.
// This is due to the fact that some values (objects) do not resolve to strings.
var valueDescArr:Array = ["{}",'"string"','"10"',"null","undefined","true","false","0","1","-1","1.23","-1.23","NaN","Infinity","emptyFunction",
                            "emptyObject","myClassA","myClassB","myClassC","myDate","Number.MAX_VALUE",'Number.NEGATIVE_INFINITY','Number.POSITIVE_INFINITY',
                            'uint.MAX_VALUE','uint.MIN_VALUE','int.MAX_VALUE','int.MIN_VALUE','"" (empty string)'];

var typeArr:Array =     [String,Number,int,uint,Boolean,Object, Function,TestClassA,TestClassB,TestClassC,TestInterface,Date];
var typeDescArr:Array = ["String","Number","int","uint","Boolean","Object","Function","TestClassA","TestClassB","TestClassC","TestInterface","Date"];

// The resultArr Array holds the expected boolean values when each value is compared to type using "is"
var resultArr  = new Array(typeArr.length);

var x:int = 0;  //counter for resultArr.  DO NOT change the line order of the array.

//      [String,    Number,     int,    uint,   Boolean,Object,         Func,   TCA,        TCB,        TCC,    TInterface, Date];
resultArr[x++] =[null,      null,       null,   null,   null,{},        null,    null,      null,       null,   null,       null];      // {}
resultArr[x++] =["string",  null,       null,   null,   null,"string",          null,    null,      null,       null,   null,       null];      // "string"
resultArr[x++] =["10",      null,       null,   null,   null,   "10",           null,    null,      null,       null,   null,       null];      // "10"
resultArr[x++] =[null,      null,       null,   null,   null,   null,           null,    null,      null,       null,   null,       null];      // null
resultArr[x++] =[null,      null,       null,   null,   null,   null,           null,    null,      null,       null,   null,       null];      // undefined
resultArr[x++] =[null,      null,       null,   null,   true,  true,            null,    null,      null,       null,   null,       null];      // true
resultArr[x++] =[null,      null,       null,   null, false,  false,            null,    null,      null,       null,   null,       null];      // false
resultArr[x++] =[null,      0,          0,      0,  null,   0,          null,    null,      null,       null,   null,       null];      // 0
resultArr[x++] =[null,      1,          1,      1,  null,   1,          null,    null,      null,       null,   null,       null];      // 1
resultArr[x++] =[null,      -1,         -1,     null,   null,   -1,         null,    null,      null,       null,   null,       null];      // -1
resultArr[x++] =[null,      1.23,       null,   null,   null,   1.23,           null,    null,      null,       null,   null,       null];      // 1.23
resultArr[x++] =[null,     -1.23,       null,   null,   null,  -1.23,           null,    null,      null,       null,   null,       null];      // -1.23
resultArr[x++] =[null,      NaN,        null,   null,   null,   NaN,            null,    null,      null,       null,   null,       null];      // NaN
resultArr[x++] =[null,      Infinity,       null,   null,   null, Infinity,         null,    null,      null,       null,   null,       null];      // Infinity
resultArr[x++] =[null,      null,       null,   null,   null,function Function() {},function Function() {},null,null,       null,   null,       null];      // emptyFunction
resultArr[x++] =[null,      null,       null,   null,   null,'[object Object]',     null,    null,      null,       null,   null,       null];      // emptyObject
resultArr[x++] =[null,      null,       null,   null,   null,'[object TestClassA]',   null,'[object TestClassA]',   null,       null,   null,       null];      // myClassA
resultArr[x++] =[null,      null,       null,   null,   null,'[object TestClassB]',   null,'[object TestClassB]','[object TestClassB]',null,    null,       null];      // myClassB
resultArr[x++] =[null,      null,       null,   null,   null,'[object TestClassC]',   null,'[object TestClassC]','[object TestClassC]','[object TestClassC]','[object TestClassC]',null];   // myClassC
resultArr[x++] =[null,      null,       null,   null,   null,'Sat Sep 24 00:00:00 GMT-0700 1977',           null,    null,      null,       null,   null,'Sat Sep 24 00:00:00 GMT-0700 1977'];      // myDate
resultArr[x++] =[null,1.79769313486231e+308,null,   null,   null,1.79769313486231e+308, null,    null,      null,       null,   null,       null];      // Number.MAX_VALUE
resultArr[x++] =[null,      -Infinity,      null,   null,   null,   -Infinity,          null,    null,      null,       null,   null,       null];      // Number.NEGATIVE_INFINITY
resultArr[x++] =[null,      Infinity,       null,   null,   null,   Infinity,           null,    null,      null,       null,   null,       null];      // Number.POSITIVE_INFINITY
resultArr[x++] =[null,      4294967295,     null,   4294967295, null,   4294967295,         null,    null,      null,       null,   null,       null];      // uint.MAX_VALUE
resultArr[x++] =[null,      0,          0,      0,  null,   0,          null,    null,      null,       null,   null,       null];      // uint.MIN_VALUE
resultArr[x++] =[null,      2147483647,     2147483647,2147483647,null, 2147483647,     null,    null,      null,       null,   null,       null];      // int.MAX_VALUE
resultArr[x++] =[null,      -2147483648,    -2147483648,null,   null,   -2147483648,            null,    null,      null,       null,   null,       null];      // int.MIN_VALUE
resultArr[x++] =["",        null,       null,   null,   null,   "",         null,    null,      null,       null,   null,       null];      // "" (empty string)

var typeArrLength = typeArr.length;

//var result:String = "";

for (var i:int = 0; i < valueArr.length; i++) {
    for (var j:int = 0; j < typeArrLength; j++) {
        try {
        var result = (valueArr[i] as typeArr[j]);
        if (resultArr[i][j] === result) {  //answer is the same as resultArray
            result = true;
        } else if  (String(resultArr[i][j]) == String(result))  { //if the answers didn't match check with a string comparison
            result = true;
        }

        } catch (e) {
        result = e;
        } finally {
        if (valueArr[i]!=myDate || (typeArr[j]!=Date && typeArr[j]!=Object))
          Assert.expectEq("("+valueDescArr[i]+" as "+ typeDescArr[j]+") Expected: "+String(resultArr[i][j]),true,result);
        }

    }
}

////////////////////////////////////////////////////////////////

              // displays results.
