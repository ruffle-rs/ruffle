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


// as Array
Assert.expectEq( "null as Array as Array", null, (null as Array as Array));

// as Boolean
Assert.expectEq( "null as Boolean as Boolean", null, (null as Boolean as Boolean));

// as Date
Assert.expectEq( "null as Date as Date", null, (null as Date as Date));

// as Function
Assert.expectEq( "null as Function as Function", null, (null as Function as Function));

// as Math
Assert.expectEq( "null as Math as Math", null, (null as Math as Math));

// as Number
Assert.expectEq( "null as Number as Number", null, (null as Number as Number));

// as Object
Assert.expectEq( "null as Object as Object", null, (null as Object as Object));

// as RegExp
Assert.expectEq( "null as RegExp as RegExp", null, (null as RegExp as RegExp));

// as String
Assert.expectEq( "null as String as String", null, (null as String as String));

// as int
Assert.expectEq( "null as int as int", null, (null as int as int));

// as uint
Assert.expectEq( "null as uint as uint", null, (null as uint as uint));

// as void
//Assert.expectEq( "null as void as void", undefined, (null as void as void));

//other
Assert.expectEq( "null as Array as Boolean as Date as Number as String ", null,
             (null as Array as Boolean as Date as Number as String ));

              // displays results.


