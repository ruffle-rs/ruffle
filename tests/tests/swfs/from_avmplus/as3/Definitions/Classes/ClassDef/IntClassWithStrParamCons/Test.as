/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testInternalClassWithParamCons.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Constructors with parameters of a Dynamic class";  // Provide ECMA section                                                                   //title or a description
var BUGNUMBER = "";



//var x = "test";
//var y:Boolean = true;
//var myArray:Array = new Array(4,6,5);
//var pbClCons:publicClassCons = new publicClassCons();
var WIntCWPC:wrapIntClassWithStrParamCons=new wrapIntClassWithStrParamCons();
//print (dynWithStrParamCons.myString());
//print(x);
//print (dynWithStrParamCons.myBoolean());
//print(y);
//print (dynWithStrParamCons.myarray());
//print (myArray);
//print(dynWithStrParamCons.myAdd());

Assert.expectEq("calling public Instance method","test",WIntCWPC.wrapmyString());
Assert.expectEq("calling public Instance method", true,WIntCWPC.wrapmyBoolean());

Assert.expectEq("Array", WIntCWPC.myArray1(),WIntCWPC.wrapmyarray());
Assert.expectEq("Calling public Instance method Add",9,WIntCWPC.wrapmyAdd());




              // displays results.
