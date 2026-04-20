/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "as operator";        // Provide ECMA section title or a description
var BUGNUMBER = "";


Assert.expectEq( "'string' as String", "string", ("string" as String));
Assert.expectEq( "'string' as Number", null, ("string" as Number));
Assert.expectEq( "'string' as int", null, ("string" as int));
Assert.expectEq( "'string' as uint", null, ("string" as uint));
Assert.expectEq( "'string' as Boolean", null, ("string" as Boolean));
Assert.expectEq( "'string' as Object", "string", ("string" as Object));
Assert.expectEq( "null as String", null, (null as String));
Assert.expectEq( "null as Number", null, (null as Number)); // bug 141358
Assert.expectEq( "null as int", null, (null as int));
Assert.expectEq( "null as uint", null, (null as uint));
Assert.expectEq( "null as Boolean", null, (null as Boolean));
Assert.expectEq( "null as Object", null, (null as Object));
Assert.expectEq( "undefined as String", null, (undefined as String)); // bug 131810
Assert.expectEq( "undefined as Number", null, (undefined as Number));
Assert.expectEq( "undefined as int", null, (undefined as int));
Assert.expectEq( "undefined as uint", null, (undefined as uint));
Assert.expectEq( "undefined as Boolean", null, (undefined as Boolean));
Assert.expectEq( "undefined as Object", null, (undefined as Object));
Assert.expectEq( "true as String", null, (true as String)); // bug 131810
Assert.expectEq( "true as Number", null, (true as Number));
Assert.expectEq( "true as int", null, (true as int));
Assert.expectEq( "true as uint", null, (true as uint));
Assert.expectEq( "true as Boolean", true, (true as Boolean));
Assert.expectEq( "true as Object", true, (true as Object));
Assert.expectEq( "false as String", null, (false as String));
Assert.expectEq( "false as Number", null, (false as Number));
Assert.expectEq( "false as int", null, (false as int));
Assert.expectEq( "false as uint", null, (false as uint));
Assert.expectEq( "false as Boolean", false, (false as Boolean));
Assert.expectEq( "false as Object", false, (false as Object));
Assert.expectEq( "1.23 as String", null, (1.23 as String));
Assert.expectEq( "1.23 as Number", 1.23, (1.23 as Number));
Assert.expectEq( "1.23 as int", null, (1.23 as int));
Assert.expectEq( "1.23 as uint", null, (1.23 as uint));
Assert.expectEq( "1.23 as Boolean", null, (1.23 as Boolean));
Assert.expectEq( "1.23 as Object", 1.23, (1.23 as Object));
Assert.expectEq( "-1.23 as String", null, (-1.23 as String));
Assert.expectEq( "-1.23 as Number", -1.23, (-1.23 as Number));
Assert.expectEq( "-1.23 as int", null, (-1.23 as int));
Assert.expectEq( "-1.23 as uint", null, (-1.23 as uint));
Assert.expectEq( "-1.23 as Boolean", null, (-1.23 as Boolean));
Assert.expectEq( "-1.23 as Object", -1.23, (-1.23 as Object));
Assert.expectEq( "NaN as String", null, (NaN as String));
Assert.expectEq( "NaN as Number", NaN, (NaN as Number));
Assert.expectEq( "NaN as int", null, (NaN as int));
Assert.expectEq( "NaN as uint", null, (NaN as uint));
Assert.expectEq( "NaN as Boolean", null, (NaN as Boolean));
Assert.expectEq( "NaN as Object", NaN, (NaN as Object));

              // displays results.


