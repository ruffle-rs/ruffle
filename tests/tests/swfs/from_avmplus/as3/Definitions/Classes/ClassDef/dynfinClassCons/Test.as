/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testdynfinalClassCons.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Constructors of a dynamic class without parameters";  // Provide ECMA section                                                                     //title or a description
var BUGNUMBER = "";



//var currentDate = new Date();
var deffinCons:dynfinClassCons = new dynfinClassCons();
//print (deffinCons.Add());
//print (DefCons.wrapprivchangeval());
//print (DefCons.wrapprotmystring());
//print (DefCons.currentdate());
//print (DefCons.wrapintmyobject());
//print (DefCons.mydatatype);

Assert.expectEq("calling public Instance method",60,deffinCons.Add());





              // displays results.
