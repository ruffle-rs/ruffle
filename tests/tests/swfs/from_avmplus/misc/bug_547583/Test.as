/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


var results = []

// there are two entry points to lastIndexOf from AS3, depending
// on whether we can early-bind the call or not; be sure to test both
// (requires compiling with -AS3)
function runTestTyped():void
{
    var str:String = "Gero";
    for (var i:int = 0; i < str.length; i++) {
        var val:int = str.lastIndexOf(str.charAt(i), -1);
        results.push({expected: -1, actual:val});
    }
}
function runTestUntyped()
{
    var str = "Gero";
    var val;
    for (var i = 0; i < str.length; i++) {
        var val:int = str.lastIndexOf(str.charAt(i), -1);
        results.push({expected: -1, actual:val});
    }
}

runTestTyped();
runTestUntyped();

for (var i in results)
{
    var o = results[i]
    Assert.expectEq("test_"+i, o.expected, o.actual);
}
