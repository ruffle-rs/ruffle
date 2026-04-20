/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

class Foo
{
    public function Foo():void;
};

function runtest()
{
    var e = "failure";
    try
    {
        var f = new Foo("bad");
    }
    catch (exception:ArgumentError)
    {
        e = "success";
    }
    catch (exception:*)
    {
        e = "failure";
    }


    Assert.expectEq("unchecked", "success", e);

}

runtest();
