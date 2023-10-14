/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "";
// var VERSION = "ECMA_1";
// var TITLE   = "interfaceMethodReturnType";


var t:TestClass = new TestClass();

Assert.expectEq("return Vector.<Object>", "obj", t.vo[0].toString() );
Assert.expectEq("return Vector.<String>", "str", t.vs[0].toString() );
Assert.expectEq("return Vector.<C>", "[object C]", t.vc[0].toString() );
Assert.expectEq("return Vector.<int>", "7", t.vi[0].toString() );
Assert.expectEq("return Vector.<*>", "any", t.va[0].toString() );
