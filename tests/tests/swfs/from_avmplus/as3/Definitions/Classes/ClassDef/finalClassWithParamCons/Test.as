/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testfinalClassWithParamCons.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Constructors with parameters of a final class";  // Provide ECMA section                                                                 // title or a                                                                 // description
var BUGNUMBER = "";


//print("test");
                     
var finalWithParamCons = new finalClassWithParamCons(20,40);
//print (finalWithParamCons.Add());


Assert.expectEq("calling public Instance method",60,finalWithParamCons.Add());





              // displays results.
