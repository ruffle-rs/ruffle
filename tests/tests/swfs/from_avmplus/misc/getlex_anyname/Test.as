/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

    import com.adobe.test.Assert;
    result = "";
    function run_test()
    {
        try
        {
            var x = 10;
            // bug being tested caused an assert to fire if OP_getlex was given an anyname.
            // thus only debug builds would have "failed".
            // (the looking is expected to throw)
            var s = String(public::*);
            result = "FAILED";
        }
        catch(e)
        {
            result = "PASSED";
        }
    }
    run_test();
    Assert.expectEq("String(public::*) should throw an exception", "PASSED", result);

