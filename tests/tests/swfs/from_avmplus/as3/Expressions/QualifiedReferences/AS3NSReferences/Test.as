/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// var SECTION = "Directives";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS 3.0";        // Version of ECMAScript or ActionScript
// var TITLE   = "Namespace values";       // Provide ECMA section title or a description
var BUGNUMBER = "";



import ns.*;
var f:foo = new foo();
import com.adobe.test.Assert;

Assert.expectEq("f.N1::A()", "www.ecma-international.org", f.N1::A());
Assert.expectEq("f.N3::flower1", "Gerbera Daisy", f.N3::flower1);
Assert.expectEq("f.N4::flower2", "Rose", f.N4::flower2);

