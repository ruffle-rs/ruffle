/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import DynamicClassMethPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Add function to Dynamic Class";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var Obj = new DynamicClassMeth();

Obj.foo = function () { return 100;}


// ********************************************
// Access property of type Number
//
// ********************************************

Assert.expectEq( "*** Access function added to a dynamic class ***", 1, 1 );
Assert.expectEq( "Obj.foo()", 100, Obj.foo() );


              // displays results.
