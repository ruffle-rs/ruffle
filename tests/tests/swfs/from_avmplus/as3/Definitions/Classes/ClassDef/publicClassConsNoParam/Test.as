/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import publicClassConsNoParamPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Constructors with no parameters of a public class";  // Provide ECMA section                                                                     //title or a description
var BUGNUMBER = "";



                     
var NoParamCons:publicClassConsNoParam = new publicClassConsNoParam();
//print (NoParamCons.Add());


Assert.expectEq("calling public  Instance method",30,NoParamCons.Add());





              // displays results.
