/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=526295
*
*/
//-----------------------------------------------------------------------------

    import com.adobe.test.Utils;
    import com.adobe.test.Assert;
    class BadCode {

        public var state:Object;

        public function BadCode() {
            state = new Object();
            state.x = 1;
        }

        // Calling this code will throw the Error object thrown in the throwAnError() method
        public final function goodCode():void {
            try {
                throwAnError();
                state.x = state.x - 1;
            }
            finally {
            }
        }

        // Call this code will not throw the Error, instead a "undefined" exception is thrown from this
        // method directly.
        public final function badCode():void {
            try {
                throwAnError();
                state.x--;
            }
            finally {
            }
        }

        public final function throwAnError():void {
            throw new Error();
        }
    }

    err = "no error";
    var foo:BadCode = new BadCode();
    try {
        foo.goodCode();
    } catch (e) {
        err = Utils.grabError(e, e.toString());
        Assert.expectEq("goodCode", "Error", err );
    }


    err = "no error";
    try {
        foo.badCode();
    } catch (e) {
        err = Utils.grabError(e, e.toString());
        Assert.expectEq("badCode", "Error", err );
    }




