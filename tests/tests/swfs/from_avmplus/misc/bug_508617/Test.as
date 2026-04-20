/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

class A
{
    private const p1:Object = new Object();
    protected const p2:Object = new Object();
    public const p3:Object = new Object();

    public function A()
    {
    }
}

class B extends A
{
    public function B()
    {
        super();
    }
}

class C1 extends A
{
    public function C1()
    {
        super();
        this.p1 = null; // illegal, should throw an error
    }
}

class C2 extends A
{
    public function C2()
    {
        super();
        this.p2 = null; // illegal, should throw an error
    }
}

class C3 extends A
{
    public function C3()
    {
        super();
        this.p3 = null; // illegal, should throw an error
    }
}

class C4 extends A
{
    public function C4()
    {
        super();
        this["p3"] = null; // illegal, should throw an error
    }
}

function test_one(c)
{
    var e = "no-exception";
    try
    {
        var x = new c();
    }
    catch (exception:*)
    {
        e = String(exception).substr(0, 27);
    }
    return e;
}


    var results = []

    results.push({expected: "no-exception", actual: test_one(B)});
    results.push({expected: "ReferenceError: Error #1056", actual: test_one(C1)});
    results.push({expected: "ReferenceError: Error #1074", actual: test_one(C2)});
    results.push({expected: "ReferenceError: Error #1074", actual: test_one(C3)});
    results.push({expected: "ReferenceError: Error #1074", actual: test_one(C4)});

for (var i in results)
{
    var o = results[i]
    Assert.expectEq("test_"+i, o.expected, o.actual);
}

