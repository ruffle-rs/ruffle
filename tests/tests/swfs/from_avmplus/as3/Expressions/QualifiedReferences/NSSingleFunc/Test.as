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
var f:foo = new foo();

Assert.expectEq( "f.Baseball::getTeam()", "Giants", f.Baseball::getTeam() );


Assert.expectEq( "f.Hockey::getTeam()", "Sharks", f.Hockey::getTeam() );

use namespace Basketball;
Assert.expectEq( "use namespace Basketball, f.getTeam()", "Kings", f.getTeam() );


              // displays results.
