/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


    var a:Array = new Array;
    a[258] = "foo";
    a[259] = "bar";

    var results:Array = new Array;

    for (var idx:String in a)
    {
        results.push(idx);
        results.push(a[idx]);
    }

    for each (var v:String in a)
    {
        results.push(v);
    }

    Assert.expectEq(0, "258", String(results[0]));
    Assert.expectEq(1, "foo", results[1]);
    Assert.expectEq(2, "259", String(results[2]));
    Assert.expectEq(3, "bar", results[3]);
    Assert.expectEq(4, "foo", results[4]);
    Assert.expectEq(5, "bar", results[5]);


