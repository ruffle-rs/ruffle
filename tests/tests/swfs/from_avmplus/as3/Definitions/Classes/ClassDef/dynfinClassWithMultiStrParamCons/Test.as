/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testdynfinalClassWithMultiStringParamCons.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Constructors with parameters of a Dynamic class";  // Provide ECMA section                                                                   //title or a description
var BUGNUMBER = "";



var x = "test";
var y = "hello"+"world";
var dynWithStrParamCons= new dynfinClassWithMultiStrParamCons(x);
var g2=new dynfinClassWithMultiStrParamCons(x+y);
//print(dynWithStrParamCons.myString());
//print(dynWithStrParamCons.myString2());
//print(g2.myString());
Assert.expectEq("calling public Instance method for testing constructor with single string parameter","test",dynWithStrParamCons.myString());
Assert.expectEq("calling public Instance method for testing constructor with concatenated string","helloworld",dynWithStrParamCons.myString2());
Assert.expectEq("calling public Instance method for testing constructor with concatenated string","testhelloworld",g2.myString());


              // displays results.
