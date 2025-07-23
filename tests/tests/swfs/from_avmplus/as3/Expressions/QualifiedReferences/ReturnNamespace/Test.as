/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS 3.0";        // Version of ECMAScript or ActionScript
// var TITLE   = "qualified references";       // Provide ECMA section title or a description
var BUGNUMBER = "";




import ns.*;

import com.adobe.test.Assert;
var b:B = new B();
var a:A = new A();


Assert.expectEq("Ref to NS returned from another class", "making my day", b.befriendAnA(a));


              // displays results.
