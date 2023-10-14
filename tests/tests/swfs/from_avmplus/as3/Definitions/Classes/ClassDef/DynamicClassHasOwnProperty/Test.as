/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import DynamicClassHasOwnPropertyPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Call hasOwnProperty() on a property of a dynamic class";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var Obj:DynamicClassHasOwnProperty = new DynamicClassHasOwnProperty();


// ********************************************
// Call hasOwnProperty()
//
// ********************************************

Assert.expectEq( "Obj.hasOwnProperty(\"bar\")", true, Obj.hasOwnProperty("bar") );


              // displays results.
