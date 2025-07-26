/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// This will cause assertions in debug builds

    import flash.utils.*;
import com.adobe.test.Assert;
    function foo() {
        var o1:Object = new Object();
        var o2:Object = new Dictionary();
        return (o1 in o2);
    }

    Assert.expectEq("This testcase would assert in debug if bug is present. o1 in o2:", false, foo());


