/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


class Foo {
    public static function failsVerification(o:Object):void
    {
        if (o && (o is Array || o.hasOwnProperty("length")) && o.length) {
            // whatever
        }
    }
}
Foo.failsVerification({});

Assert.expectEq("This test will fail verification above if bug is present", true, true);

