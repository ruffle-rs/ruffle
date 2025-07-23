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
Assert.expectEq( "null as Array", null, (null as Array));
Assert.expectEq( "[1,1,1,1] as Array", "1,1,1,1", ([1,1,1,1] as Array).toString());
Assert.expectEq( "var array = new Array('1'); array as Array", "1", (array = new Array('1'), (array as Array).toString()));
Assert.expectEq( "Boolean as Array", null, (Boolean as Array));
Assert.expectEq( "Date as Array", null, (Date as Array));
Assert.expectEq( "Function as Array", null, (Function as Array));
Assert.expectEq( "Math as Array", null, (Math as Array));
Assert.expectEq( "Number as Array", null, (Number as Array));
Assert.expectEq( "Object as Array", null, (Object as Array));
Assert.expectEq( "String as Array", null, (String as Array));
Assert.expectEq( "RegExp as Array", null, (RegExp as Array));
Assert.expectEq( "int as Array", null, (int as Array));
Assert.expectEq( "uint as Array", null, (uint as Array));

// as Boolean
Assert.expectEq( "null as Boolean", null, (null as Boolean));
Assert.expectEq( "false as Boolean", false, (false as Boolean));
Assert.expectEq( "true as Boolean", true, (true as Boolean));
Assert.expectEq( "Array as Boolean", null, (Array as Boolean));
Assert.expectEq( "Date as Boolean", null, (Date as Boolean));
Assert.expectEq( "Function as Boolean", null, (Function as Boolean));
Assert.expectEq( "Math as Boolean", null, (Math as Boolean));
Assert.expectEq( "Number as Boolean", null, (Number as Boolean));
Assert.expectEq( "Object as Boolean", null, (Object as Boolean));
Assert.expectEq( "String as Boolean", null, (String as Boolean));
Assert.expectEq( "RegExp as Boolean", null, (RegExp as Boolean));
Assert.expectEq( "int as Boolean", null, (int as Boolean));
Assert.expectEq( "uint as Boolean", null, (uint as Boolean));

// as Date
Assert.expectEq( "null as Date", null, (null as Date));
Assert.expectEq( "undefined as Date", null, (undefined as Date));
Assert.expectEq( "'test' as Date", null, ('test' as Date));
// fails if not PDT
//Assert.expectEq( "new Date(0) as Date", "Wed Dec 31 16:00:00 GMT-0800 1969", (new Date(0) as Date).toString());
Assert.expectEq( "'Wed Dec 31 16:00:00 GMT-0800 1969' as Date", null, ("Wed Dec 31 16:00:00 GMT-0800 1969" as Date));
Assert.expectEq( "Array as Date", null, (Array as Date));
Assert.expectEq( "Boolean as Date", null, (Boolean as Date));
Assert.expectEq( "Function as Date", null, (Function as Date));
Assert.expectEq( "Math as Date", null, (Math as Date));
Assert.expectEq( "Number as Date", null, (Number as Date));
Assert.expectEq( "Object as Date", null, (Object as Date));
Assert.expectEq( "String as Date", null, (String as Date));
Assert.expectEq( "''as Date", null, ("" as Date));
Assert.expectEq( "RegExp as Date", null, (RegExp as Date));
Assert.expectEq( "int as Date", null, (int as Date));
Assert.expectEq( "uint as Date", null, (uint as Date));
//Assert.expectEq( "void as Date", null, (void as Date));
Assert.expectEq( "Date as Date", null, (Date as Date));



// as Function
Assert.expectEq( "null as Function ", null, (null as Function));

// as Math
Assert.expectEq( "null as Math", null, (null as Math));

// as Number
Assert.expectEq( "null as Number", null, (null as Number));
Assert.expectEq( "1 as Number", 1, (1 as Number));
Assert.expectEq( "1.66 as Number", 1.66, (1.66 as Number));
Assert.expectEq( "-1.66 as Number", -1.66, (-1.66 as Number));

// as Object
Assert.expectEq( "null as Object", null, (null as Object));

// as RegExp
Assert.expectEq( "null as RegExp", null, (null as RegExp));

// as String
Assert.expectEq( "null as String", null, (null as String));
Assert.expectEq( "undefined as String", null, (undefined as String));
Assert.expectEq( "'' as String", "undefined", ("undefined" as String));
Assert.expectEq( "'foo' as String", "foo", ('foo' as String));
Assert.expectEq( "new String('foo') as String", "foo", (new String('foo') as String));
Assert.expectEq( "Array as String", null, (Array as String));
Assert.expectEq( "Boolean as String", null, (Boolean as String));
Assert.expectEq( "new Boolean(true) as String", null, (new Boolean(true) as String));
Assert.expectEq( "Date as String", null, (Date as String));
Assert.expectEq( "Function as String", null, (Function as String));
Assert.expectEq( "Math as String", null, (Math as String));
Assert.expectEq( "Number as String", null, (Number as String));
Assert.expectEq( "Object as String", null, (Object as String));
Assert.expectEq( "RegExp as String", null, (RegExp as String));
Assert.expectEq( "int as String", null, (int as String));
Assert.expectEq( "uint as String", null, (uint as String));
//Assert.expectEq( "void as String", "[class void]", (void as String));

// as int
Assert.expectEq( "null as int", null, (null as int));
Assert.expectEq( "0 as int", 0, (0 as int));
Assert.expectEq( "1 as int", 1, (1 as int));
Assert.expectEq( "-1 as int", -1, (-1 as int));

// as uint
Assert.expectEq( "null as uint", null, (null as uint));
Assert.expectEq( "0 as uint", 0, (0 as uint));
Assert.expectEq( "1 as uint", 1, (1 as uint));
Assert.expectEq( "100 as uint", 100, (100 as uint));
Assert.expectEq( "-1 as uint", null, (-1 as uint));

// as void
//Assert.expectEq( "null as void", undefined, (null as void));


              // displays results.


