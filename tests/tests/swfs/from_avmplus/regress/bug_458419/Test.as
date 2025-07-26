/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;
//     var SECTION="";
//     var VERSION = "";




    function f():String
    {
        var v : Vector.<String> = Vector.<String>(["one", "two"]);
        var ans:String = "";

        if( 5.5 in v )
            ans = "5.5 in v";
        else
            ans = "5.5 not in v";

        return ans
    }

    Assert.expectEq("hasAtomProperty for Vector should not throw an exception",
                "5.5 not in v",
                f()
                );

