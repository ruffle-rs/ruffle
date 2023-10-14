/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Types: Conversions";
// var VERSION = "as3";
// var TITLE   = "type conversions";


// "string"
Assert.expectEq("String('string')", "string", String("string"));
Assert.expectEq("String('')", "", String(""));
Assert.expectEq("Number('string')", NaN, Number("string"));
Assert.expectEq("Number('')", 0, Number(""));
Assert.expectEq("int('string')", 0, int("string"));
Assert.expectEq("int('')", 0, int(""));
Assert.expectEq("uint('string')", 0, uint("string"));
Assert.expectEq("uint('')", 0, uint(""));
Assert.expectEq("Boolean('string')", true, Boolean("string"));
Assert.expectEq("Boolean('')", false, Boolean(""));
Assert.expectEq("Object('string')", "string", Object("string"));
Assert.expectEq("Object('')", "", Object(""));

// null

Assert.expectEq( "String(null)", "null", String(null));
Assert.expectEq( "Number(null)", +0, Number(null));
Assert.expectEq( "int(null)", +0, int(null));
Assert.expectEq( "uint(null)", +0, uint(null));
Assert.expectEq( "Boolean(null)", false, Boolean(null));
Assert.expectEq( "Object(null)", "[object Object]", Object(null)+"");

// undefined
Assert.expectEq( "String(undefined)", "undefined", String(undefined));
Assert.expectEq( "Number(undefined)", NaN, Number(undefined));
Assert.expectEq( "int(undefined)", +0, int(undefined));
Assert.expectEq( "uint(undefined)", +0, uint(undefined));
Assert.expectEq( "Boolean(undefined)", false, Boolean(undefined));
Assert.expectEq( "Object(undefined)", "[object Object]", Object(undefined)+"");

// true
Assert.expectEq( "String(true)", "true", String(true));
Assert.expectEq( "Number(true)", 1, Number(true));
Assert.expectEq( "int(true)", 1, int(true));
Assert.expectEq( "uint(true)", 1, uint(true));
Assert.expectEq( "Boolean(true)", true, Boolean(true));
Assert.expectEq( "Object(true)", true, Object(true));

// false
Assert.expectEq( "String(false)", "false", String(false));
Assert.expectEq( "Number(false)", +0, Number(false));
Assert.expectEq( "int(false)", +0, int(false));
Assert.expectEq( "uint(false)", +0, uint(false));
Assert.expectEq( "Boolean(false)", false, Boolean(false));
Assert.expectEq( "Object(false)", false, Object(false));

// 1.23
Assert.expectEq( "String(1.23)", "1.23", String(1.23));
Assert.expectEq( "Number(1.23)", 1.23, Number(1.23));
Assert.expectEq( "int(1.23)", 1, int(1.23));
Assert.expectEq( "uint(1.23)", 1, uint(1.23));
Assert.expectEq( "Boolean(1.23)", true, Boolean(1.23));
Assert.expectEq( "Object(1.23)", 1.23, Object(1.23));

// -1.23
Assert.expectEq( "String(-1.23)", "-1.23", String(-1.23));
Assert.expectEq( "Number(-1.23)", -1.23, Number(-1.23));
Assert.expectEq( "int(-1.23)", -1, int(-1.23));
Assert.expectEq( "uint(-1.23)", 4294967295, uint(-1.23));
Assert.expectEq( "Boolean(-1.23)", true, Boolean(-1.23));
Assert.expectEq( "Object(-1.23)", -1.23, Object(-1.23));

// NaN
Assert.expectEq( "String(NaN)", "NaN", String(NaN));
Assert.expectEq( "Number(NaN)", NaN, Number(NaN));
Assert.expectEq( "int(NaN)", +0, int(NaN));
Assert.expectEq( "uint(NaN)", +0, uint(NaN));
Assert.expectEq( "Boolean(NaN)", false, Boolean(NaN));
Assert.expectEq( "Object(NaN)", NaN, Object(NaN));





