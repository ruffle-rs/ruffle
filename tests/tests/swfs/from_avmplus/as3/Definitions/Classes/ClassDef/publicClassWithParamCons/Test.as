/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testpublicClassWithParamCons.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "No Parameter Constructors of a Internal class";  // Provide ECMA section                                                                 // title or a description
var BUGNUMBER = "";



var a:Number=20;
var b:Number=0.5;

                     
var publicWithParamCons:publicClassWithParamCons = new publicClassWithParamCons(a,b);
//print (publicWithParamCons.Add());

Assert.expectEq("calling public Instance method",20.5,publicWithParamCons.Add());





              // displays results.
