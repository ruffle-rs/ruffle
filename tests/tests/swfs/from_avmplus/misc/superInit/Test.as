/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

class Thing
{
}

class A
{
    public var container:Thing;
}

class B extends A
{
    public function B(s:Thing)
    {
        /*
            existing code relies on the fact that for object-typed member slots,
            we don't generate explicit initialization code for the slot as part
            of the ctor, but rather just leave the slot alone (it's allocated
            pre-zeroed-out)... this allows assignments to inherited members to
            work when done prior to the super call.
        */
        container = s;  // yes, assign *before* the super calls
        super();
    }
}

var t = new Thing;
var b:B = new B(t);


Assert.expectEq("String(b.container)", "[object Thing]", String(b.container));

