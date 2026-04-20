/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "as operator";        // Provide ECMA section title or a description
var BUGNUMBER = "";


var string:String = "string" as String;
Assert.expectEq( "var string:String = 'string' as String", "string", string);

var number:Number = "string" as Number;
Assert.expectEq( "var number:Number = 'string' as Number", 0, number);

var myint:int = "string" as int;
Assert.expectEq( "var myint:int = 'string' as int", +0, myint);

var myuint:uint = "string" as uint;
Assert.expectEq( "var myuint:uint = 'string' as uint", +0, myuint);

var boolean:Boolean = "string" as Boolean;
Assert.expectEq( "var boolean:Boolean = 'string' as Boolean", false, boolean);

var object:Object = "string" as Object;
Assert.expectEq( "var object:Object = 'string' as Object", "string", object);

var string2:String = null as String;
Assert.expectEq( "var string:String = null as String", null, string2);

var number2:Number = null as Number;
Assert.expectEq( "var number:Number = null as Number", 0, number2);

var myint2:int = null as int;
Assert.expectEq( "var myint:int = null as int", +0, myint2);

var myuint2:uint = null as uint;
Assert.expectEq( "var myuint2:uint = null as uint", +0, myuint2);

var boolean2:Boolean = null as Boolean;
Assert.expectEq( "var boolean2:Boolean = null as Boolean", false, boolean2);

var object2:Object = null as Object;
Assert.expectEq( "var object2:Object = null as Object", null, object2);

var string3:String = undefined as String;
Assert.expectEq( "var string3:String = undefined as String", null, string3); // bug 131810

var number3:Number = undefined as Number;
Assert.expectEq( "var number3:Number = undefined as Number", 0, number3);

var myint3:int = undefined as int;
Assert.expectEq( "var myint3:int = undefined as int", +0, myint3);

var myuint3:uint = undefined as uint;
Assert.expectEq( "var myuint3:uint = undefined as uint", +0, myuint3);

var boolean3:Boolean = undefined as Boolean;
Assert.expectEq( "var boolean3:Boolean = undefined as Boolean", false, boolean3);

var object3:Object = undefined as Object;
Assert.expectEq( "var object3:Object = undefined as Object", null, object3);

var string4:String = true as String;
Assert.expectEq( "var string4:String = true as String", null, string4); // bug 131810

var number4:Number = true as Number;
Assert.expectEq( "var number4:Number = true as Number", 0, number4);

var myint4:int = true as int;
Assert.expectEq( "var myint4:int = true as int", 0, myint4);

var myuint4:uint = true as uint;
Assert.expectEq( "var myuint4:uint = true as uint", 0, myuint4);

var boolean4:Boolean = true as Boolean;
Assert.expectEq( "var boolean4:Boolean = true as Boolean", true, boolean4);

var object4:Object = true as Object;
Assert.expectEq( "var object4:Object = true as Object", true, object4);

var string5:String = false as String;
Assert.expectEq( "var string5:String = false as String", null, string5);

var number5:Number = false as Number;
Assert.expectEq( "var number5:Number = false as Number", 0, number5);

var myint5:int = false as int;
Assert.expectEq( "var myint5:int = false as int", +0, myint5);

var myuint5:uint = false as uint;
Assert.expectEq( "var myuint5:uint = false as uint", +0, myuint5);

var boolean5:Boolean = false as Boolean;
Assert.expectEq( "var boolean5:Boolean = false as Boolean", false, boolean5);

var object5:Object = false as Object;
Assert.expectEq( "var object5:Object = false as Object", false, object5);

var string6:String = 1.23 as String;
Assert.expectEq( "var string6:String = 1.23 as String", null, string6);

var number6:Number = 1.23 as Number;
Assert.expectEq( "var number6:Number = 1.23 as Number", 1.23, number6);

var myint6:int = 1.23 as int;
Assert.expectEq( "var myint6:int = 1.23 as int", 0, myint6);

var myuint6:uint = 1.23 as uint;
Assert.expectEq( "var myuint6:uint = 1.23 as uint", 0, myuint6);

var boolean6:Boolean = 1.23 as Boolean;
Assert.expectEq( "var boolean6:Boolean = 1.23 as Boolean", false, boolean6);

var object6:Object = 1.23 as Object;
Assert.expectEq( "var object6:Object = 1.23 as Object", 1.23, object6);

var string7:String = -1.23 as String;
Assert.expectEq( "var string7:String = -1.23 as String", null, string7);

var number7:Number = -1.23 as Number;
Assert.expectEq( "var number7:Number = -1.23 as Number", -1.23, number7);

var myint7:int = -1.23 as int;
Assert.expectEq( "var myint7:int = -1.23 as int", 0, myint7);

var myuint7:uint = -1.23 as uint;
Assert.expectEq( "var myuint7:uint = -1.23 as uint", 0, myuint7);

var boolean7:Boolean = -1.23 as Boolean;
Assert.expectEq( "var boolean7:Boolean = -1.23 as Boolean", false, boolean7);

var object7:Object = -1.23 as Object;
Assert.expectEq( "var object7:Object = -1.23 as Object", -1.23, object7);

var string8:String = NaN as String;
Assert.expectEq( "var string8:String = NaN as String", null, string8);

var number8:Number = NaN as Number;
Assert.expectEq( "var number8:Number = NaN as Number", NaN, number8);

var myint8:int = NaN as int;
Assert.expectEq( "var myint8:int = NaN as int", +0, myint8);

var myuint8:uint = NaN as uint;
Assert.expectEq( "var myuint8:uint = NaN as uint", +0, myuint8);

var boolean8:Boolean = NaN as Boolean;
Assert.expectEq( "var boolean8:Boolean = NaN as Boolean", false, boolean8);

var object8:Object = NaN as Object;
Assert.expectEq( "var object8:Object = NaN as Object", NaN, object8);


              // displays results.


