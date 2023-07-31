/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import publicClassConstructors.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Default Constructors of a public class";  // Provide ECMA section title or a description
var BUGNUMBER = "";



var currentDate = new Date(0);
var DefCons:publicClassDefCons = new publicClassDefCons();
//print (DefCons.Add());
//print (DefCons.wrapprivchangeval());
//print (DefCons.wrapprotmystring());
//print (DefCons.currentdate());
//print (DefCons.wrapintmyobject());
//print (DefCons.mydatatype);

Assert.expectEq("calling public final Instance method",30,DefCons.Add());
Assert.expectEq("Calling private Instance method",false,DefCons.wrapprivchangeval());
Assert.expectEq("Calling public Instance method",currentDate.toString(),(DefCons.currentdate()).toString());
Assert.expectEq("Calling internal Instance method","I am a string",DefCons.wrapintmyobject());





              // displays results.
