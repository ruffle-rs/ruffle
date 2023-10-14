/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import StatClassSameNamePackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Access Static Properties & Methods";  // Provide ECMA section title or a description
var BUGNUMBER = "134955";




Assert.expectEq( "Access static property via package/class with same name", "x.x.a", StatClassSameNamePackage.StatClassSameNamePackage.aStatic );
Assert.expectEq( "Call static method via package/class with same name", "x.x.f()", StatClassSameNamePackage.StatClassSameNamePackage.fStatic() );


              // displays results.
