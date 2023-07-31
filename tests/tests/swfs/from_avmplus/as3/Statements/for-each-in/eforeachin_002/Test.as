/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Statements";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "for each in";       // Provide ECMA section title or a description
var BUGNUMBER = "";


 
 // XML Object
 
 var i, s="";
 
 obj1 = {};
 obj1.A = 1;
 
 obj2 = {};
 obj2.A = 2;
 obj3 = {};
 obj3.A = 3;
 
 x1 = {};
 x1.z = [obj1,obj2,obj3];
 
// var xmlDoc = "<L><z><A>1</A></z><z><A>2</A></z><z><A>3</A></z></L>";
// var x1 = new XML(xmlDoc);
 
 for each(var i in x1.z) {
       s += i.A;
 }
 
 Assert.expectEq( "for-each-in (var) :", true, (s=="123") );
 
 s = 0;
 
 for each ( i in x1.z )
    s += i.A;
 
 Assert.expectEq( "for-each-in       :", true, (s==6) );


              // displays results.
