/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}



/*
    This file is the original source for bug_555705.abc_

    bug_555705.abc_ was created by changing offset 18c from 00 to 01 in the
    generated abc for this file (asc.jar version 14710).

*/



    function f() {
        print("Hello, world!");
    }
    function g() {
        print("Bug 555705 - should not leak memory: PASSED!");
    }
    class C {
        function hello() {
            try {
                f();
            }
            catch (e1 : RangeError) {
                print("Range");
            }
            catch (e2) {
                if (e2.toString().substr(0,24) == "VerifyError: Error #1025") {
                    print("Modified abc throws 'VerifyError: Error #1025: An invalid register 1 was accessed.' PASSED!");
                } else {
                    print("Unexpected Error Thrown FAILED! : "+e2.toString());
                }
            }
            finally {
                g();
            }
        }
    }
    (new C).hello();

