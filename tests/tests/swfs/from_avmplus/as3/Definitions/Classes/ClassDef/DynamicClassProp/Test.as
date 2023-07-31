/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import DynamicClassPropPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Add Properties to Dynamic Class";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var Obj = new DynamicClassProp();


var arr = new Array(1,2,3);

var d = new Date(0);

var str = "Test";

var ob = new Object();


Obj.prop1 = 100;
Obj.prop2 = "Test";
Obj.prop3 = true;
Obj.prop4 = arr;
Obj.prop5 = d;
Obj.prop6 = ob;


// ********************************************
// Access property of type Number
//
// ********************************************

Assert.expectEq( "*** Access property of type Number ***", 1, 1 );
Assert.expectEq( "Obj.prop1", 100, Obj.prop1 );


// ********************************************
// Access property of type String
//
// ********************************************

Assert.expectEq( "*** Access property of type String ***", 1, 1 );
Assert.expectEq( "Obj.prop2", "Test", Obj.prop2 );


// ********************************************
// Access property of type Boolean
//
// ********************************************

Assert.expectEq( "*** Access property of type Boolean ***", 1, 1 );
Assert.expectEq( "Obj.prop3", true, Obj.prop3 );


// ********************************************
// Access property of type Array
//
// ********************************************

Assert.expectEq( "*** Access property of type Array ***", 1, 1 );
Assert.expectEq( "Obj.prop4", arr, Obj.prop4 );

// ********************************************
// Access property of type Date
// ********************************************

Assert.expectEq( "*** Access property of type Date ***", 1, 1 );
Assert.expectEq( "Obj.prop5", d, Obj.prop5 );


// ********************************************
// Access property of type Object
// ********************************************

Assert.expectEq( "*** Access property of type Object ***", 1, 1 );
Assert.expectEq( "Obj.prop6", ob, Obj.prop6 );



              // displays results.
