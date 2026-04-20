/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
class Base
{
    public var x, y;
    public var intField:int;
}

class TestSuper extends Base
{
    public var intField2:int;
    public function testSuperPostInc()
    {
        Assert.expectEq("typeof super.x++", "number", typeof super.x++);
        Assert.expectEq("super.x", Number.NaN, super.x);
        Assert.expectEq("super.intField++", 0, super.intField++);
        Assert.expectEq("typeof super.intField", "number", typeof super.intField);
        Assert.expectEq("super.intField", 1, super.intField);
        Assert.expectEq("intField2++", 0, intField2++);
        Assert.expectEq("intField2", 1, intField2);
    }

    public function testSuperPostDec()
    {
        Assert.expectEq("typeof super.x--", "number", typeof super.x--);
        Assert.expectEq("super.x", Number.NaN, super.x);
        Assert.expectEq("super.intField--", 0, super.intField--);
        Assert.expectEq("typeof super.intField", "number", typeof super.intField);
        Assert.expectEq("super.intField", -1, super.intField);
        Assert.expectEq("intField2--", 0, intField2--);
        Assert.expectEq("intField2", -1, intField2);
    }
}

// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Postfix operators";       // Provide ECMA section title or a description

/*
  Note that this test is an extention to the following ecma3 tests:
  ecma3/Expressions/e11_3_1.as
  ecma3/Expressions/e11_3_2.as
*/


new TestSuper().testSuperPostInc();
new TestSuper().testSuperPostDec();

              // displays results.