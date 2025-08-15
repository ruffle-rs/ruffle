/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import MultipleExtraArgFunction2.*
import com.adobe.test.Assert;

function returnRestNoPackage(str:String,n:Number,obj:Object,... rest):Number {
    var count = rest.length;
    var a:int = 0;
    return count;
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Optional Argument test";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ = new TestObj();
var TESTOBJ1 = new MultipleExtraArgFunction2Class();

// inside class inside package
Assert.expectEq( "TESTOBJ.returnRest()", 6, TESTOBJ.returnRest([20,30],40,10,false,"hello",new Object(), [123],[345]) );

// inside package outside of class
Assert.expectEq( "returnRest()", 7, returnRest("Str",[1,2,3],[10,11,12],false,"hello",new Object(), [123],[345],"hello") );

// outside package inside class
Assert.expectEq( "TESTOBJ1.returnRest()", 6, TESTOBJ1.returnRest(new Object(),[1,2,3],10,new Object(),"hello",new Object(), [123],[345]) );

// outside package and outside class
Assert.expectEq( "returnRestNoPackage()", 6, returnRestNoPackage("Str",1000,new Object(),10,"str2","hello",new Object(), [123],[345]) );


              // displays results.
